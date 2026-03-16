use std::{
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::{
    glib, pango, prelude::*, Box as GtkBox, Button, EventControllerKey, Label, ListBox,
    Orientation, Paned, PolicyType, ScrolledWindow, SelectionMode, TextView, WrapMode,
};

use super::{
    ops::{self, DiffHunk},
    GitPaneView,
};

// ─── Data model ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum HunkStatus {
    #[default]
    Unreviewed,
    Reviewed,
    Risky,
    FollowUp,
}

impl HunkStatus {
    fn icon(&self) -> &'static str {
        match self {
            HunkStatus::Unreviewed => "\u{00B7}",   // ·
            HunkStatus::Reviewed => "\u{2713}",      // ✓
            HunkStatus::Risky => "\u{2691}",         // ⚑
            HunkStatus::FollowUp => "?",
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            HunkStatus::Unreviewed => "status-unreviewed",
            HunkStatus::Reviewed => "status-reviewed",
            HunkStatus::Risky => "status-risky",
            HunkStatus::FollowUp => "status-followup",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            HunkStatus::Unreviewed => "unreviewed",
            HunkStatus::Reviewed => "reviewed",
            HunkStatus::Risky => "risky",
            HunkStatus::FollowUp => "follow-up",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PatchHunk {
    file: String,
    hunk_index: usize,
    header: String,
    diff_text: String,
    annotation: String,
    status: HunkStatus,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct PatchSession {
    branch: String,
    hunks: Vec<PatchHunk>,
    commit_scope_draft: String,
}

// ─── Persistence ─────────────────────────────────────────────────────

fn data_root() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("magma")
}

fn session_path(repo_root: &Path, branch: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_root.hash(&mut hasher);
    let repo_hash = format!("{:016x}", hasher.finish());
    let sanitized = branch.replace('/', "_");
    data_root()
        .join("patch")
        .join(repo_hash)
        .join(format!("{sanitized}.json"))
}

fn load_session(repo_root: &Path, branch: &str) -> Option<PatchSession> {
    let data = std::fs::read_to_string(session_path(repo_root, branch)).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_session(repo_root: &Path, session: &PatchSession) {
    let path = session_path(repo_root, &session.branch);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(&path, data);
    }
}

fn clear_session_file(repo_root: &Path, branch: &str) {
    let _ = std::fs::remove_file(session_path(repo_root, branch));
}

// ─── Widgets ─────────────────────────────────────────────────────────

pub(super) struct PatchWidgets {
    pub(super) hunk_list: ListBox,
    state: Rc<PatchState>,
}

struct PatchState {
    session: RefCell<PatchSession>,
    selected: Cell<usize>,
    /// Guards against recursive saves when `update_detail` sets annotation text.
    suppress_annotation_save: Cell<bool>,

    // Widgets
    detail_box: GtkBox,
    annotation_view: TextView,
    commit_draft_label: Label,
    hunk_list: ListBox,

    /// Parallel to `session.hunks` — direct references to the status icon labels
    /// in the hunk list, avoiding expensive tree walks on status change.
    status_labels: RefCell<Vec<Label>>,
}

// ─── Build ───────────────────────────────────────────────────────────

pub(super) fn build_patch_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-patch-root");
    root.set_vexpand(true);

    // ─── Split pane: hunk queue (left) | detail (right) ─────────
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_vexpand(true);
    paned.set_position(180);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.add_css_class("magma-git-patch-paned");

    // Left: hunk queue
    let left = GtkBox::new(Orientation::Vertical, 0);
    left.add_css_class("magma-git-patch-queue");
    left.set_hexpand(false);

    let hunk_list = ListBox::new();
    hunk_list.set_selection_mode(SelectionMode::None);
    hunk_list.add_css_class("magma-git-patch-list");

    let left_scroller = ScrolledWindow::new();
    left_scroller.set_vexpand(true);
    left_scroller.set_policy(PolicyType::Never, PolicyType::Automatic);
    left_scroller.set_child(Some(&hunk_list));

    left.append(&left_scroller);

    // Right: detail + annotation
    let right = GtkBox::new(Orientation::Vertical, 0);
    right.add_css_class("magma-git-patch-detail-root");
    right.set_hexpand(true);

    let detail_box = GtkBox::new(Orientation::Vertical, 0);
    detail_box.add_css_class("magma-git-patch-detail");

    let detail_scroller = ScrolledWindow::new();
    detail_scroller.set_vexpand(true);
    detail_scroller.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    detail_scroller.set_child(Some(&detail_box));

    // Annotation area
    let annotation_label = Label::new(Some("annotation"));
    annotation_label.add_css_class("magma-git-patch-annotation-label");
    annotation_label.set_xalign(0.0);

    let annotation_view = TextView::new();
    annotation_view.set_wrap_mode(WrapMode::WordChar);
    annotation_view.add_css_class("magma-git-patch-annotation");
    annotation_view.set_top_margin(4);
    annotation_view.set_bottom_margin(4);
    annotation_view.set_left_margin(6);
    annotation_view.set_right_margin(6);

    let annotation_frame = GtkBox::new(Orientation::Vertical, 0);
    annotation_frame.add_css_class("magma-git-patch-annotation-frame");
    annotation_frame.append(&annotation_view);

    // Status marking row
    let status_row = GtkBox::new(Orientation::Horizontal, 4);
    status_row.add_css_class("magma-git-patch-status-row");

    let mark_reviewed = mark_button("\u{2713} Reviewed", "mark-reviewed", "Mark as reviewed (r)");
    let mark_risky = mark_button("\u{2691} Risky", "mark-risky", "Mark as risky (x)");
    let mark_followup = mark_button("? Follow-up", "mark-followup", "Mark for follow-up (f)");
    let mark_reset = mark_button("Reset", "", "Reset to unreviewed (u)");

    status_row.append(&mark_reviewed);
    status_row.append(&mark_risky);
    status_row.append(&mark_followup);
    status_row.append(&mark_reset);

    right.append(&detail_scroller);
    right.append(&annotation_label);
    right.append(&annotation_frame);
    right.append(&status_row);

    paned.set_start_child(Some(&left));
    paned.set_end_child(Some(&right));

    // ─── Bottom action bar ──────────────────────────────────────
    let action_bar = GtkBox::new(Orientation::Horizontal, 4);
    action_bar.add_css_class("magma-git-patch-actions");

    let commit_draft_label = Label::new(None);
    commit_draft_label.add_css_class("magma-git-patch-draft-label");
    commit_draft_label.set_xalign(0.0);
    commit_draft_label.set_hexpand(true);
    commit_draft_label.set_ellipsize(pango::EllipsizeMode::End);

    let draft_btn = Button::builder()
        .label("draft scope")
        .css_classes(["magma-git-action-btn"])
        .tooltip_text("Draft commit scope from reviewed hunks (d)")
        .build();

    let clear_btn = Button::builder()
        .label("clear queue")
        .css_classes(["magma-git-action-btn"])
        .tooltip_text("Clear all hunks and annotations")
        .build();

    let export_btn = Button::builder()
        .label("export")
        .css_classes(["magma-git-action-btn"])
        .tooltip_text("Export patch brief (e)")
        .build();

    action_bar.append(&commit_draft_label);
    action_bar.append(&draft_btn);
    action_bar.append(&clear_btn);
    action_bar.append(&export_btn);

    root.append(&paned);
    root.append(&action_bar);

    // ─── State ──────────────────────────────────────────────────
    let state = Rc::new(PatchState {
        session: RefCell::new(PatchSession::default()),
        selected: Cell::new(0),
        suppress_annotation_save: Cell::new(false),
        detail_box,
        annotation_view: annotation_view.clone(),
        commit_draft_label: commit_draft_label.clone(),
        hunk_list: hunk_list.clone(),
        status_labels: RefCell::new(Vec::new()),
    });

    // ─── Annotation change → save (guarded) ─────────────────────
    {
        let state_ref = state.clone();
        let view_ref = view.clone();
        annotation_view.buffer().connect_changed(move |buf| {
            if state_ref.suppress_annotation_save.get() {
                return;
            }
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
            let idx = state_ref.selected.get();
            let mut session = state_ref.session.borrow_mut();
            if let Some(hunk) = session.hunks.get_mut(idx) {
                hunk.annotation = text;
            }
            if let Some(root) = view_ref.repo_root.borrow().as_ref() {
                save_session(root, &session);
            }
        });
    }

    // ─── Status marking buttons ─────────────────────────────────
    bind_mark_button(&mark_reviewed, HunkStatus::Reviewed, &state, view);
    bind_mark_button(&mark_risky, HunkStatus::Risky, &state, view);
    bind_mark_button(&mark_followup, HunkStatus::FollowUp, &state, view);
    bind_mark_button(&mark_reset, HunkStatus::Unreviewed, &state, view);

    // ─── Draft commit scope ─────────────────────────────────────
    {
        let state_ref = state.clone();
        let view_ref = view.clone();
        draft_btn.connect_clicked(move |_| {
            draft_commit_scope(&state_ref, &view_ref);
        });
    }

    // ─── Clear queue ────────────────────────────────────────────
    {
        let state_ref = state.clone();
        let view_ref = view.clone();
        clear_btn.connect_clicked(move |_| {
            {
                let session = state_ref.session.borrow();
                if let Some(root) = view_ref.repo_root.borrow().as_ref() {
                    clear_session_file(root, &session.branch);
                }
            }
            {
                let mut session = state_ref.session.borrow_mut();
                session.hunks.clear();
                session.commit_scope_draft.clear();
            }
            state_ref.selected.set(0);
            rebuild_hunk_list(&state_ref);
            clear_detail(&state_ref);
            state_ref.commit_draft_label.set_text("");
            view_ref.set_status_ok("patch queue cleared");
        });
    }

    // ─── Export ─────────────────────────────────────────────────
    {
        let state_ref = state.clone();
        let view_ref = view.clone();
        export_btn.connect_clicked(move |_| {
            export_patch_brief(&state_ref, &view_ref);
        });
    }

    // ─── Keyboard shortcuts (local to patch view) ───────────────
    bind_patch_keys(&root, &state, view);

    *view.patch_widgets.borrow_mut() = Some(PatchWidgets {
        hunk_list: hunk_list.clone(),
        state: state.clone(),
    });

    root
}

fn mark_button(label: &str, extra_class: &str, tooltip: &str) -> Button {
    let btn = Button::builder()
        .label(label)
        .css_classes(["magma-git-patch-mark-btn"])
        .tooltip_text(tooltip)
        .build();
    if !extra_class.is_empty() {
        btn.add_css_class(extra_class);
    }
    btn
}

// ─── Refresh ─────────────────────────────────────────────────────────

pub(super) fn refresh_patch(view: &Rc<GitPaneView>) {
    let widgets = view.patch_widgets.borrow();
    let Some(widgets) = widgets.as_ref() else {
        return;
    };

    let root = view.repo_root.borrow().clone();
    let Some(root) = root else {
        return;
    };

    // Branch is already populated by the caller (refresh → git_status),
    // read it from the header label to avoid a redundant git call.
    let branch = view.branch_label.text().to_string();

    let file_hunks = match ops::git_diff_all_hunks(&root) {
        Ok(fh) => fh,
        Err(e) => {
            view.set_status_err(&format!("patch diff failed: {e}"));
            return;
        }
    };

    // Load saved session and merge annotations/statuses into fresh hunks
    let saved = load_session(&root, &branch);
    let mut hunks: Vec<PatchHunk> = Vec::new();

    for (file, diff_hunks) in &file_hunks {
        for (hunk_index, hunk) in diff_hunks.iter().enumerate() {
            let diff_text = format_hunk_text(hunk);
            let (annotation, status) = saved
                .as_ref()
                .and_then(|s| {
                    s.hunks.iter().find(|h| {
                        h.file == *file && h.hunk_index == hunk_index && h.header == hunk.header
                    })
                })
                .map(|h| (h.annotation.clone(), h.status))
                .unwrap_or_default();

            hunks.push(PatchHunk {
                file: file.clone(),
                hunk_index,
                header: hunk.header.clone(),
                diff_text,
                annotation,
                status,
            });
        }
    }

    let draft = saved.map(|s| s.commit_scope_draft).unwrap_or_default();

    {
        let mut session = widgets.state.session.borrow_mut();
        session.branch = branch;
        session.hunks = hunks;
        session.commit_scope_draft = draft.clone();
    }

    widgets.state.commit_draft_label.set_text(&draft);
    widgets.state.selected.set(0);
    rebuild_hunk_list(&widgets.state);

    // Show detail for first hunk (or empty state)
    update_detail(&widgets.state);

    let count = widgets.state.session.borrow().hunks.len();
    if count == 0 {
        view.set_status("no changes to review");
    } else {
        view.set_status(&format!("{count} hunks"));
    }
}

// ─── Keyboard ────────────────────────────────────────────────────────

fn bind_patch_keys(root: &GtkBox, state: &Rc<PatchState>, view: &Rc<GitPaneView>) {
    let key_ctrl = EventControllerKey::new();
    let state_ref = state.clone();
    let view_ref = view.clone();

    key_ctrl.connect_key_pressed(move |_, keyval, _keycode, _modifier| {
        // Don't intercept keys when annotation has focus
        if state_ref.annotation_view.has_focus() {
            // Escape leaves the annotation field
            if keyval == gtk::gdk::Key::Escape {
                state_ref.hunk_list.grab_focus();
                return glib::Propagation::Stop;
            }
            return glib::Propagation::Proceed;
        }

        match keyval {
            gtk::gdk::Key::j | gtk::gdk::Key::Down => {
                move_selection(&state_ref, 1);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::k | gtk::gdk::Key::Up => {
                move_selection(&state_ref, -1);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::Return | gtk::gdk::Key::a => {
                state_ref.annotation_view.grab_focus();
                glib::Propagation::Stop
            }
            gtk::gdk::Key::r => {
                set_current_status(&state_ref, &view_ref, HunkStatus::Reviewed);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::x => {
                set_current_status(&state_ref, &view_ref, HunkStatus::Risky);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::f => {
                set_current_status(&state_ref, &view_ref, HunkStatus::FollowUp);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::u => {
                set_current_status(&state_ref, &view_ref, HunkStatus::Unreviewed);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::d => {
                draft_commit_scope(&state_ref, &view_ref);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::e => {
                export_patch_brief(&state_ref, &view_ref);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });

    root.add_controller(key_ctrl);
}

// ─── Internal helpers ────────────────────────────────────────────────

fn bind_mark_button(
    btn: &Button,
    status: HunkStatus,
    state: &Rc<PatchState>,
    view: &Rc<GitPaneView>,
) {
    let state_ref = state.clone();
    let view_ref = view.clone();
    btn.connect_clicked(move |_| {
        set_current_status(&state_ref, &view_ref, status);
    });
}

fn set_current_status(state: &Rc<PatchState>, view: &Rc<GitPaneView>, status: HunkStatus) {
    let idx = state.selected.get();
    {
        let mut session = state.session.borrow_mut();
        let Some(hunk) = session.hunks.get_mut(idx) else {
            return;
        };
        hunk.status = status;
        if let Some(root) = view.repo_root.borrow().as_ref() {
            save_session(root, &session);
        }
    }
    update_row_status(state, idx);
    view.set_status(&format!("marked {}", status.label()));
}

fn move_selection(state: &Rc<PatchState>, delta: i32) {
    let count = state.session.borrow().hunks.len();
    if count == 0 {
        return;
    }
    let current = state.selected.get() as i32;
    let next = (current + delta).clamp(0, count as i32 - 1) as usize;
    if next == state.selected.get() {
        return;
    }
    state.selected.set(next);

    // Update visual selection
    highlight_selected_row(state);
    update_detail(state);
}

fn rebuild_hunk_list(state: &Rc<PatchState>) {
    clear_container(&state.hunk_list);

    let session = state.session.borrow();
    let mut labels = Vec::with_capacity(session.hunks.len());
    let mut current_file = "";

    if session.hunks.is_empty() {
        let empty = Label::new(Some("no changes to review"));
        empty.add_css_class("magma-git-empty");
        empty.set_xalign(0.0);
        state.hunk_list.append(&empty);
        *state.status_labels.borrow_mut() = labels;
        return;
    }

    for (i, hunk) in session.hunks.iter().enumerate() {
        // File group header
        if hunk.file != current_file {
            current_file = &hunk.file;
            let file_label = Label::new(Some(current_file));
            file_label.add_css_class("magma-git-patch-file-header");
            file_label.set_xalign(0.0);
            file_label.set_ellipsize(pango::EllipsizeMode::Start);
            state.hunk_list.append(&file_label);
        }

        // Hunk row
        let row = GtkBox::new(Orientation::Horizontal, 4);
        row.add_css_class("magma-git-patch-hunk-row");
        if i == state.selected.get() {
            row.add_css_class("selected");
        }

        let status_icon = Label::new(Some(hunk.status.icon()));
        status_icon.add_css_class("magma-git-patch-hunk-status");
        status_icon.add_css_class(hunk.status.css_class());

        let hunk_label = Label::new(Some(&format!("hunk {}", hunk.hunk_index + 1)));
        hunk_label.add_css_class("magma-git-patch-hunk-label");
        hunk_label.set_xalign(0.0);
        hunk_label.set_hexpand(true);
        hunk_label.set_ellipsize(pango::EllipsizeMode::End);

        row.append(&status_icon);
        row.append(&hunk_label);

        // Click to select
        let state_ref = state.clone();
        let gesture = gtk::GestureClick::new();
        let hunk_idx = i;
        gesture.connect_released(move |_, _, _, _| {
            state_ref.selected.set(hunk_idx);
            highlight_selected_row(&state_ref);
            update_detail(&state_ref);
        });
        row.add_controller(gesture);

        state.hunk_list.append(&row);
        labels.push(status_icon);
    }

    *state.status_labels.borrow_mut() = labels;
}

fn highlight_selected_row(state: &Rc<PatchState>) {
    // Walk children and toggle `.selected` on hunk rows
    let selected = state.selected.get();
    let mut hunk_idx = 0usize;
    let mut child = state.hunk_list.first_child();
    while let Some(ref current) = child {
        if current.has_css_class("magma-git-patch-hunk-row") {
            if hunk_idx == selected {
                current.add_css_class("selected");
            } else {
                current.remove_css_class("selected");
            }
            hunk_idx += 1;
        }
        child = current.next_sibling();
    }
}

fn update_row_status(state: &Rc<PatchState>, idx: usize) {
    let session = state.session.borrow();
    let Some(hunk) = session.hunks.get(idx) else {
        return;
    };
    let labels = state.status_labels.borrow();
    let Some(label) = labels.get(idx) else {
        return;
    };
    label.set_text(hunk.status.icon());
    for class in &["status-unreviewed", "status-reviewed", "status-risky", "status-followup"] {
        label.remove_css_class(class);
    }
    label.add_css_class(hunk.status.css_class());
}

fn update_detail(state: &Rc<PatchState>) {
    clear_container_box(&state.detail_box);

    let session = state.session.borrow();
    let idx = state.selected.get();
    let Some(hunk) = session.hunks.get(idx) else {
        let empty = Label::new(Some("no hunk selected"));
        empty.add_css_class("magma-git-empty");
        state.detail_box.append(&empty);
        state.suppress_annotation_save.set(true);
        state.annotation_view.buffer().set_text("");
        state.suppress_annotation_save.set(false);
        return;
    };

    // File + hunk header
    let file_label = Label::new(Some(&hunk.file));
    file_label.add_css_class("magma-git-patch-detail-file");
    file_label.set_xalign(0.0);
    file_label.set_ellipsize(pango::EllipsizeMode::Start);
    state.detail_box.append(&file_label);

    if !hunk.header.is_empty() {
        let header_label = Label::new(Some(&hunk.header));
        header_label.add_css_class("magma-git-hunk-header-text");
        header_label.set_xalign(0.0);
        header_label.set_ellipsize(pango::EllipsizeMode::End);
        state.detail_box.append(&header_label);
    }

    // Diff lines
    for line in hunk.diff_text.lines() {
        let line_label = Label::new(Some(line));
        line_label.set_xalign(0.0);
        line_label.set_selectable(true);
        line_label.add_css_class("magma-git-diff-line");
        line_label.add_css_class(diff_line_class(line));
        state.detail_box.append(&line_label);
    }

    // Update annotation without triggering save
    state.suppress_annotation_save.set(true);
    state.annotation_view.buffer().set_text(&hunk.annotation);
    state.suppress_annotation_save.set(false);
}

fn diff_line_class(line: &str) -> &'static str {
    if line.starts_with('+') {
        "magma-git-line-added"
    } else if line.starts_with('-') {
        "magma-git-line-removed"
    } else {
        "magma-git-line-context"
    }
}

fn clear_detail(state: &Rc<PatchState>) {
    clear_container_box(&state.detail_box);
    state.suppress_annotation_save.set(true);
    state.annotation_view.buffer().set_text("");
    state.suppress_annotation_save.set(false);
}

fn format_hunk_text(hunk: &DiffHunk) -> String {
    let mut text = String::new();
    for (i, line) in hunk.lines.iter().enumerate() {
        if i > 0 {
            text.push('\n');
        }
        text.push_str(&line.content);
    }
    text
}

fn draft_commit_scope(state: &Rc<PatchState>, view: &Rc<GitPaneView>) {
    let (draft, count) = {
        let session = state.session.borrow();
        let reviewed: Vec<&PatchHunk> = session
            .hunks
            .iter()
            .filter(|h| h.status == HunkStatus::Reviewed)
            .collect();

        if reviewed.is_empty() {
            view.set_status("no reviewed hunks to draft");
            return;
        }

        let mut draft = String::from("Changes:\n");
        let mut current_file = "";
        for hunk in &reviewed {
            if hunk.file != current_file {
                current_file = &hunk.file;
                draft.push_str(&format!("\n  {current_file}:\n"));
            }
            if hunk.annotation.is_empty() {
                draft.push_str(&format!("    hunk {}\n", hunk.hunk_index + 1));
            } else {
                let first_line = hunk.annotation.lines().next().unwrap_or("");
                draft.push_str(&format!("    hunk {}: {first_line}\n", hunk.hunk_index + 1));
            }
        }

        (draft, reviewed.len())
    };

    state
        .commit_draft_label
        .set_text(draft.lines().next().unwrap_or(""));

    state.session.borrow_mut().commit_scope_draft = draft;

    if let Some(root) = view.repo_root.borrow().as_ref() {
        save_session(root, &state.session.borrow());
    }
    view.set_status_ok(&format!("drafted scope ({count} reviewed hunks)"));
}

fn export_patch_brief(state: &Rc<PatchState>, view: &Rc<GitPaneView>) {
    let (out, branch) = {
        let session = state.session.borrow();
        if session.hunks.is_empty() {
            view.set_status("nothing to export");
            return;
        }

        let mut out = format!("# Patch Brief: {}\n\n", session.branch);
        let mut current_file = "";

        for hunk in &session.hunks {
            if hunk.file != current_file {
                current_file = &hunk.file;
                out.push_str(&format!("\n## {current_file}\n\n"));
            }
            out.push_str(&format!(
                "### Hunk {} [{}]\n",
                hunk.hunk_index + 1,
                hunk.status.label()
            ));
            if !hunk.header.is_empty() {
                out.push_str(&format!("`{}`\n", hunk.header));
            }
            if !hunk.annotation.is_empty() {
                out.push_str(&format!(
                    "\n> {}\n",
                    hunk.annotation.replace('\n', "\n> ")
                ));
            }
            out.push('\n');
        }

        if !session.commit_scope_draft.is_empty() {
            out.push_str(&format!(
                "\n---\n\n## Commit Scope Draft\n\n{}\n",
                session.commit_scope_draft
            ));
        }

        (out, session.branch.replace('/', "_"))
    };

    let repo_root = view.repo_root.borrow().clone();
    if let Some(root) = repo_root {
        let export_path = root.join(format!("patch-brief-{branch}.md"));
        match std::fs::write(&export_path, &out) {
            Ok(_) => view.set_status_ok(&format!("exported to {}", export_path.display())),
            Err(e) => view.set_status_err(&format!("export failed: {e}")),
        }
    }
}

// ─── Container clearing ─────────────────────────────────────────────

fn clear_container(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn clear_container_box(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
