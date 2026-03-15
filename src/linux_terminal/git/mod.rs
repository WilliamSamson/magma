#[allow(dead_code)]
mod branches;
#[allow(dead_code)]
mod diff;
#[allow(dead_code)]
mod graph;
mod host;
#[allow(dead_code)]
mod ops;
#[allow(dead_code)]
mod search;
#[allow(dead_code)]
mod staging;
#[allow(dead_code)]
mod stash;

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{channel, TryRecvError},
    time::Duration,
};

use gtk::{
    gdk, glib, prelude::*, Box as GtkBox, Button, EventControllerKey, Label, Orientation,
    Stack, StackTransitionType,
};

pub(super) use host::GitPaneHost;

use super::view::CwdProvider;

// ─── Sub-view identifiers ────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubView {
    Status,
    Log,
    Branches,
    Stash,
    Search,
}

impl SubView {
    fn stack_name(&self) -> &'static str {
        match self {
            SubView::Status => "status",
            SubView::Log => "log",
            SubView::Branches => "branches",
            SubView::Stash => "stash",
            SubView::Search => "search",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SubView::Status => "status",
            SubView::Log => "log",
            SubView::Branches => "branches",
            SubView::Stash => "stash",
            SubView::Search => "search",
        }
    }
}

const SUB_VIEWS: &[SubView] = &[
    SubView::Status,
    SubView::Log,
    SubView::Branches,
    SubView::Stash,
    SubView::Search,
];

// ─── Shared pane state ───────────────────────────────────────────────

pub(super) struct GitPaneView {
    root: GtkBox,
    cwd_provider: CwdProvider,
    repo_root: RefCell<Option<PathBuf>>,
    branch_label: Label,
    ahead_behind_label: Label,
    status_label: Label,
    nav_stack: Stack,
    active_view: RefCell<SubView>,
    nav_buttons: Vec<Button>,

    // Remote action buttons
    fetch_btn: Button,
    pull_btn: Button,
    push_btn: Button,

    // Sub-view widget refs (populated by each sub-view builder)
    staging_widgets: RefCell<Option<Rc<staging::StagingWidgets>>>,
    graph_widgets: RefCell<Option<Rc<graph::GraphState>>>,
    branch_widgets: RefCell<Option<branches::BranchWidgets>>,
    stash_widgets: RefCell<Option<stash::StashWidgets>>,
    search_widgets: RefCell<Option<Rc<search::SearchWidgets>>>,
}

impl GitPaneView {
    fn set_status(&self, text: &str) {
        self.clear_status_style();
        self.status_label.set_text(text);
    }

    fn set_status_ok(&self, text: &str) {
        self.clear_status_style();
        self.status_label.set_text(text);
        self.status_label.add_css_class("magma-git-status-ok");
    }

    fn set_status_err(&self, text: &str) {
        self.clear_status_style();
        self.status_label.set_text(text);
        self.status_label.add_css_class("magma-git-status-err");
    }

    fn set_status_busy(&self, text: &str) {
        self.clear_status_style();
        self.status_label.set_text(text);
        self.status_label.add_css_class("magma-git-status-busy");
    }

    fn clear_status_style(&self) {
        self.status_label.remove_css_class("magma-git-status-ok");
        self.status_label.remove_css_class("magma-git-status-err");
        self.status_label.remove_css_class("magma-git-status-busy");
    }

    fn set_remote_buttons_sensitive(&self, sensitive: bool) {
        self.fetch_btn.set_sensitive(sensitive);
        self.pull_btn.set_sensitive(sensitive);
        self.push_btn.set_sensitive(sensitive);
    }

    fn switch_view(&self, view: SubView) {
        *self.active_view.borrow_mut() = view;
        self.nav_stack.set_visible_child_name(view.stack_name());

        for (i, btn) in self.nav_buttons.iter().enumerate() {
            if SUB_VIEWS[i] == view {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }
    }
}

// Free function so closures can capture Rc<GitPaneView> and call it.
fn refresh(view: &Rc<GitPaneView>) {
    let cwd = (view.cwd_provider)().map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok());

    let Some(cwd) = cwd else {
        set_no_repo(view);
        return;
    };

    match ops::git_repo_root(&cwd) {
        Ok(root) => {
            *view.repo_root.borrow_mut() = Some(root.clone());

            match ops::git_status(&root) {
                Ok(status) => {
                    // Show detached HEAD or branch name
                    if status.is_detached {
                        view.branch_label.set_text(&format!("detached: {}", status.branch));
                    } else {
                        view.branch_label.set_text(&status.branch);
                    }

                    let ab = format_ahead_behind(status.ahead, status.behind);
                    view.ahead_behind_label.set_text(&ab);
                    view.ahead_behind_label.set_visible(!ab.is_empty());

                    // Disable remote buttons when no remotes configured
                    if !status.has_remotes {
                        view.set_remote_buttons_sensitive(false);
                        view.fetch_btn.set_tooltip_text(Some("No remotes configured"));
                        view.pull_btn.set_tooltip_text(Some("No remotes configured"));
                        view.push_btn.set_tooltip_text(Some("No remotes configured"));
                    } else {
                        view.set_remote_buttons_sensitive(true);
                        view.fetch_btn.set_tooltip_text(Some("Fetch all remotes"));
                        view.pull_btn.set_tooltip_text(Some("Pull from remote"));
                        view.push_btn.set_tooltip_text(Some("Push to remote"));
                    }

                    let active = *view.active_view.borrow();
                    match active {
                        SubView::Status => staging::refresh_staging(view, &status),
                        SubView::Log => graph::refresh_graph(view),
                        SubView::Branches => branches::refresh_branches(view),
                        SubView::Stash => stash::refresh_stash(view),
                        SubView::Search => {} // search refreshes on demand
                    }
                }
                Err(e) => view.set_status_err(&format!("status error: {e}")),
            }
        }
        Err(_) => set_no_repo(view),
    }
}

fn set_no_repo(view: &Rc<GitPaneView>) {
    *view.repo_root.borrow_mut() = None;
    view.branch_label.set_text("not a git repo");
    view.ahead_behind_label.set_visible(false);
    view.set_remote_buttons_sensitive(false);
    view.set_status("no git repository found");

    // Clear all sub-view lists so stale data doesn't linger
    clear_all_views(view);
}

fn clear_all_views(view: &Rc<GitPaneView>) {
    if let Some(sw) = view.staging_widgets.borrow().as_ref() {
        clear_list_box(&sw.staged_list);
        clear_list_box(&sw.unstaged_list);
        clear_list_box(&sw.untracked_list);
        clear_list_box(&sw.conflicted_list);
        sw.staged_count.set_text("0");
        sw.unstaged_count.set_text("0");
        sw.untracked_count.set_text("0");
        sw.conflicted_count.set_text("0");
        sw.conflicted_section.set_visible(false);
        sw.commit_button.set_sensitive(false);
    }
    if let Some(gw) = view.graph_widgets.borrow().as_ref() {
        clear_list_box(&gw.list);
        gw.commits.borrow_mut().clear();
        gw.load_more_btn.set_visible(false);
    }
    if let Some(bw) = view.branch_widgets.borrow().as_ref() {
        clear_list_box(&bw.local_list);
        clear_list_box(&bw.remote_list);
    }
    if let Some(sw) = view.stash_widgets.borrow().as_ref() {
        clear_list_box(&sw.list);
    }
    if let Some(sw) = view.search_widgets.borrow().as_ref() {
        clear_list_box(&sw.list);
        sw.count_label.set_text("");
    }
}

fn clear_list_box(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

// Convenience method for closures that already hold Rc<GitPaneView>
impl GitPaneView {
    fn refresh(self: &Rc<Self>) {
        refresh(self);
    }
}

// ─── Public entry point ──────────────────────────────────────────────

pub(super) fn build_git_pane(cwd_provider: CwdProvider) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.add_css_class("magma-git-root");
    root.set_focusable(true);

    // ─── Header: branch + ahead/behind ───────────────────────────
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("magma-git-header");

    let title = Label::new(Some("git"));
    title.add_css_class("magma-git-title");

    let branch_label = Label::new(Some("..."));
    branch_label.add_css_class("magma-git-branch-label");
    branch_label.set_xalign(0.0);
    branch_label.set_hexpand(true);
    branch_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let ahead_behind_label = Label::new(None);
    ahead_behind_label.add_css_class("magma-git-ahead-behind");
    ahead_behind_label.set_visible(false);

    header.append(&title);
    header.append(&branch_label);
    header.append(&ahead_behind_label);

    // ─── Remote operations bar ───────────────────────────────────
    let remote_bar = GtkBox::new(Orientation::Horizontal, 4);
    remote_bar.add_css_class("magma-git-remote-bar");

    let fetch_btn = Button::builder()
        .label("fetch")
        .css_classes(["magma-git-remote-button"])
        .tooltip_text("Fetch all remotes")
        .build();

    let pull_btn = Button::builder()
        .label("pull")
        .css_classes(["magma-git-remote-button"])
        .tooltip_text("Pull from remote")
        .build();

    let push_btn = Button::builder()
        .label("push")
        .css_classes(["magma-git-remote-button"])
        .tooltip_text("Push to remote")
        .build();

    let refresh_btn = Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["magma-git-icon-btn"])
        .tooltip_text("Refresh (Ctrl+R)")
        .build();

    remote_bar.append(&fetch_btn);
    remote_bar.append(&pull_btn);
    remote_bar.append(&push_btn);
    let remote_spacer = GtkBox::new(Orientation::Horizontal, 0);
    remote_spacer.set_hexpand(true);
    remote_bar.append(&remote_spacer);
    remote_bar.append(&refresh_btn);

    // ─── Sub-view navigation ─────────────────────────────────────
    let nav_row = GtkBox::new(Orientation::Horizontal, 2);
    nav_row.add_css_class("magma-git-nav");

    let mut nav_buttons = Vec::new();
    for sv in SUB_VIEWS {
        let btn = Button::builder()
            .label(sv.label())
            .css_classes(["magma-git-nav-button"])
            .build();
        if *sv == SubView::Status {
            btn.add_css_class("active");
        }
        nav_row.append(&btn);
        nav_buttons.push(btn);
    }

    // ─── Content stack ───────────────────────────────────────────
    let nav_stack = Stack::new();
    nav_stack.set_transition_type(StackTransitionType::Crossfade);
    nav_stack.set_transition_duration(150);
    nav_stack.set_vexpand(true);

    // ─── Status bar ──────────────────────────────────────────────
    let status_label = Label::new(Some("loading..."));
    status_label.add_css_class("magma-git-status");
    status_label.set_xalign(0.0);
    status_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    // ─── Assemble ────────────────────────────────────────────────
    root.append(&header);
    root.append(&remote_bar);
    root.append(&nav_row);
    root.append(&nav_stack);
    root.append(&status_label);

    let view = Rc::new(GitPaneView {
        root: root.clone(),
        cwd_provider,
        repo_root: RefCell::new(None),
        branch_label,
        ahead_behind_label,
        status_label,
        nav_stack: nav_stack.clone(),
        active_view: RefCell::new(SubView::Status),
        nav_buttons: nav_buttons.clone(),
        fetch_btn: fetch_btn.clone(),
        pull_btn: pull_btn.clone(),
        push_btn: push_btn.clone(),
        staging_widgets: RefCell::new(None),
        graph_widgets: RefCell::new(None),
        branch_widgets: RefCell::new(None),
        stash_widgets: RefCell::new(None),
        search_widgets: RefCell::new(None),
    });

    // Build sub-views and add to stack
    let status_view = staging::build_staging_view(&view);
    let log_view = graph::build_graph_view(&view);
    let branches_view = branches::build_branches_view(&view);
    let stash_view = stash::build_stash_view(&view);
    let search_view = search::build_search_view(&view);

    nav_stack.add_named(&status_view, Some("status"));
    nav_stack.add_named(&log_view, Some("log"));
    nav_stack.add_named(&branches_view, Some("branches"));
    nav_stack.add_named(&stash_view, Some("stash"));
    nav_stack.add_named(&search_view, Some("search"));

    // ─── Bind navigation buttons ─────────────────────────────────
    for (i, btn) in nav_buttons.iter().enumerate() {
        let view_ref = view.clone();
        let sv = SUB_VIEWS[i];
        btn.connect_clicked(move |_| {
            view_ref.switch_view(sv);
            view_ref.refresh();
        });
    }

    // ─── Bind remote operations (async — off the GTK thread) ─────
    bind_remote_op(&view, &fetch_btn, "fetching...", |repo| {
        ops::git_fetch(&repo).map(|_| "fetch complete".to_string())
    });
    bind_remote_op(&view, &pull_btn, "pulling...", |repo| {
        ops::git_pull(&repo).map(|out| {
            out.lines().next().unwrap_or("pull complete").to_string()
        })
    });
    bind_remote_op(&view, &push_btn, "pushing...", |repo| {
        ops::git_push(&repo).map(|_| "push complete".to_string())
    });

    // ─── Bind refresh button ─────────────────────────────────────
    {
        let view_ref = view.clone();
        refresh_btn.connect_clicked(move |_| {
            view_ref.refresh();
        });
    }

    // ─── Keyboard shortcuts ──────────────────────────────────────
    bind_keyboard(&view);

    // ─── Initial load + auto-refresh ─────────────────────────────
    {
        let view_ref = view.clone();
        glib::idle_add_local_once(move || {
            view_ref.refresh();
        });
    }

    watch_for_changes(&view);

    root
}

// ─── Async remote operation helper ───────────────────────────────────

fn bind_remote_op<F>(view: &Rc<GitPaneView>, btn: &Button, busy_msg: &'static str, op: F)
where
    F: Fn(PathBuf) -> Result<String, String> + Send + 'static + Clone,
{
    let view_ref = view.clone();
    let op = op.clone();
    btn.connect_clicked(move |_| {
        let repo = view_ref.repo_root.borrow().clone();
        let Some(repo) = repo else { return };

        // Lock all remote buttons + show busy status
        view_ref.set_remote_buttons_sensitive(false);
        view_ref.set_status_busy(busy_msg);

        let view_clone = view_ref.clone();
        let op = op.clone();
        let (sender, receiver) = channel();

        std::thread::spawn(move || {
            let result = op(repo);
            let _ = sender.send(result);
        });

        glib::timeout_add_local(Duration::from_millis(50), move || {
            match receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(msg) => {
                            view_clone.set_status_ok(&msg);
                            view_clone.refresh();
                        }
                        Err(e) => {
                            view_clone.set_status_err(&e);
                            // Re-enable buttons on error (refresh won't run)
                            view_clone.set_remote_buttons_sensitive(true);
                        }
                    }
                    glib::ControlFlow::Break
                }
                Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => {
                    view_clone.set_status_err("remote operation disconnected");
                    view_clone.set_remote_buttons_sensitive(true);
                    glib::ControlFlow::Break
                }
            }
        });
    });
}

// ─── Keyboard ────────────────────────────────────────────────────────

fn bind_keyboard(view: &Rc<GitPaneView>) {
    let key_ctrl = EventControllerKey::new();
    let view_ref = view.clone();

    key_ctrl.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);

        // Ctrl+R → refresh
        if ctrl && keyval == gdk::Key::r {
            view_ref.refresh();
            return glib::Propagation::Stop;
        }

        // Ctrl+F → fetch
        if ctrl && keyval == gdk::Key::f {
            view_ref.fetch_btn.emit_clicked();
            return glib::Propagation::Stop;
        }

        // 1-5 → switch sub-views
        if !ctrl {
            let sv = match keyval {
                gdk::Key::_1 => Some(SubView::Status),
                gdk::Key::_2 => Some(SubView::Log),
                gdk::Key::_3 => Some(SubView::Branches),
                gdk::Key::_4 => Some(SubView::Stash),
                gdk::Key::_5 => Some(SubView::Search),
                _ => None,
            };
            if let Some(sv) = sv {
                view_ref.switch_view(sv);
                view_ref.refresh();
                return glib::Propagation::Stop;
            }
        }

        // Escape → return to status
        if keyval == gdk::Key::Escape {
            view_ref.switch_view(SubView::Status);
            view_ref.refresh();
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    view.root.add_controller(key_ctrl);
}

// ─── Auto-refresh: cwd changes + git state changes ──────────────────

fn watch_for_changes(view: &Rc<GitPaneView>) {
    let view_ref = view.clone();
    let last_cwd: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let last_fingerprint: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    glib::timeout_add_local(Duration::from_millis(2000), move || {
        let current_cwd = (view_ref.cwd_provider)();
        let cwd_changed = {
            let last = last_cwd.borrow();
            *last != current_cwd
        };

        if cwd_changed {
            *last_cwd.borrow_mut() = current_cwd;
            *last_fingerprint.borrow_mut() = None;
            view_ref.refresh();
        } else {
            // Check if git state changed (HEAD moved, index updated, etc.)
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                let current_fp = ops::quick_repo_fingerprint(&repo);
                let fp_changed = {
                    let last = last_fingerprint.borrow();
                    *last != current_fp
                };
                if fp_changed {
                    *last_fingerprint.borrow_mut() = current_fp;
                    view_ref.refresh();
                }
            }
        }

        glib::ControlFlow::Continue
    });
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn format_ahead_behind(ahead: u32, behind: u32) -> String {
    match (ahead, behind) {
        (0, 0) => String::new(),
        (a, 0) => format!("\u{2191}{a}"),
        (0, b) => format!("\u{2193}{b}"),
        (a, b) => format!("\u{2191}{a} \u{2193}{b}"),
    }
}
