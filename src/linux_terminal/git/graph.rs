use std::{cell::RefCell, rc::Rc};

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Label, ListBox, Orientation,
    PolicyType, Revealer, RevealerTransitionType, ScrolledWindow, SelectionMode,
};

use super::{
    diff::{build_diff_stat, build_diff_widget},
    ops::{self, CommitInfo},
    GitPaneView,
};

const INITIAL_LOAD: usize = 100;
const LOAD_MORE: usize = 50;

pub(super) fn build_graph_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-graph-root");
    root.set_vexpand(true);

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("magma-git-graph-list");

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroller.set_child(Some(&list));

    let load_more_btn = Button::builder()
        .label("load more commits")
        .css_classes(["magma-git-load-more"])
        .visible(false)
        .build();

    root.append(&scroller);
    root.append(&load_more_btn);

    let graph_state = Rc::new(GraphState {
        list: list.clone(),
        commits: RefCell::new(Vec::new()),
        load_more_btn: load_more_btn.clone(),
        expanded: RefCell::new(None),
    });

    *view.graph_widgets.borrow_mut() = Some(graph_state.clone());

    // Load more button (appends without graph art — graph lines can't be
    // continued consistently after the initial batch)
    {
        let view_ref = view.clone();
        let state = graph_state.clone();
        load_more_btn.connect_clicked(move |btn| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                let skip = state.commits.borrow().len();
                btn.set_sensitive(false);
                match ops::git_log(&root, LOAD_MORE, skip) {
                    Ok(more) => {
                        let view_rc = view_ref.clone();
                        for commit in &more {
                            state.list.append(&build_commit_row(commit, &view_rc));
                        }
                        let exhausted = more.len() < LOAD_MORE;
                        state.commits.borrow_mut().extend(more);
                        state.load_more_btn.set_visible(!exhausted);
                    }
                    Err(e) => view_ref.set_status_err(&format!("load failed: {e}")),
                }
                btn.set_sensitive(true);
            }
        });
    }

    root
}

pub(super) fn refresh_graph(view: &Rc<GitPaneView>) {
    let state = view.graph_widgets.borrow();
    let Some(state) = state.as_ref() else {
        return;
    };

    clear_list(&state.list);
    state.commits.borrow_mut().clear();
    *state.expanded.borrow_mut() = None;

    let root = view.repo_root.borrow().clone();
    let Some(root) = root else {
        return;
    };

    if !ops::has_commits(&root) {
        let label = Label::new(Some("no commits yet"));
        label.add_css_class("magma-git-empty");
        label.set_xalign(0.0);
        state.list.append(&label);
        state.load_more_btn.set_visible(false);
        return;
    }

    match ops::git_log_graph(&root, INITIAL_LOAD) {
        Ok(commits) => {
            for commit in &commits {
                state.list.append(&build_commit_row(commit, view));
            }
            let has_more = commits.len() >= INITIAL_LOAD;
            state.load_more_btn.set_visible(has_more);
            *state.commits.borrow_mut() = commits;
        }
        Err(e) => {
            let label = Label::new(Some(&format!("error: {e}")));
            label.add_css_class("magma-git-error");
            label.set_xalign(0.0);
            state.list.append(&label);
        }
    }
}

fn build_commit_row(commit: &CommitInfo, view: &Rc<GitPaneView>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("magma-git-commit-container");

    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-git-commit-row");

    // Graph art (if available)
    if let Some(graph) = &commit.graph_line {
        let graph_label = Label::new(Some(graph.trim_end()));
        graph_label.add_css_class("magma-git-graph-art");
        row.append(&graph_label);
    }

    let info_box = GtkBox::new(Orientation::Vertical, 1);
    info_box.set_hexpand(true);

    // First line: hash + message + refs
    let top_row = GtkBox::new(Orientation::Horizontal, 6);

    let hash_label = Label::new(Some(&commit.short_hash));
    hash_label.add_css_class("magma-git-commit-hash");

    let msg_label = Label::new(Some(&commit.message));
    msg_label.add_css_class("magma-git-commit-msg");
    msg_label.set_xalign(0.0);
    msg_label.set_hexpand(true);
    msg_label.set_ellipsize(pango::EllipsizeMode::End);

    top_row.append(&hash_label);

    // Ref badges
    if !commit.refs.is_empty() {
        for ref_name in commit.refs.split(", ") {
            let ref_name = ref_name.trim();
            if ref_name.is_empty() {
                continue;
            }
            let badge = Label::new(Some(ref_name));
            badge.add_css_class("magma-git-ref-badge");
            if ref_name.starts_with("HEAD") {
                badge.add_css_class("ref-head");
            } else if ref_name.starts_with("tag:") {
                badge.add_css_class("ref-tag");
            }
            top_row.append(&badge);
        }
    }

    top_row.append(&msg_label);

    // Second line: author + date
    let bottom_row = GtkBox::new(Orientation::Horizontal, 8);

    let author_label = Label::new(Some(&commit.author));
    author_label.add_css_class("magma-git-commit-author");
    author_label.set_xalign(0.0);
    author_label.set_hexpand(true);
    author_label.set_ellipsize(pango::EllipsizeMode::End);

    let date_label = Label::new(Some(&commit.date));
    date_label.add_css_class("magma-git-commit-date");

    bottom_row.append(&author_label);
    bottom_row.append(&date_label);

    info_box.append(&top_row);
    info_box.append(&bottom_row);
    row.append(&info_box);

    // Expandable diff on click
    let diff_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(200)
        .build();

    let commit_hash = commit.hash.clone();
    let view_ref = view.clone();
    let revealer_ref = diff_revealer.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        if revealer_ref.reveals_child() {
            revealer_ref.set_reveal_child(false);
            return;
        }

        let root = view_ref.repo_root.borrow().clone();
        if let Some(root) = root {
            let diff_box = GtkBox::new(Orientation::Vertical, 4);
            diff_box.add_css_class("magma-git-commit-detail");

            // Show diff stat
            if let Ok(stat) = ops::git_diff_stat(&root, &commit_hash) {
                diff_box.append(&build_diff_stat(&stat));
            }

            // Show full diff
            if let Ok(hunks) = ops::git_show_diff(&root, &commit_hash) {
                diff_box.append(&build_diff_widget(&hunks, None));
            }

            revealer_ref.set_child(Some(&diff_box));
            revealer_ref.set_reveal_child(true);
        }
    });
    row.add_controller(gesture);

    container.append(&row);
    container.append(&diff_revealer);
    container
}

fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

pub(super) struct GraphState {
    pub(super) list: ListBox,
    pub(super) commits: RefCell<Vec<CommitInfo>>,
    pub(super) load_more_btn: Button,
    pub(super) expanded: RefCell<Option<String>>,
}
