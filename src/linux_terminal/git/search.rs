use std::rc::Rc;

use gtk::{
    pango, prelude::*, Box as GtkBox, DropDown, Entry, Label, ListBox, Orientation,
    PolicyType, Revealer, RevealerTransitionType, ScrolledWindow, SelectionMode, StringList,
};

use super::{
    diff::{build_diff_stat, build_diff_widget},
    ops::{self, CommitInfo, SearchMode},
    GitPaneView,
};

pub(super) fn build_search_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-search-root");
    root.set_vexpand(true);

    // Search bar
    let search_row = GtkBox::new(Orientation::Horizontal, 4);
    search_row.add_css_class("magma-git-search-bar");

    let mode_list = StringList::new(&["message", "author", "file"]);
    let mode_dropdown = DropDown::new(Some(mode_list), None::<gtk::Expression>);
    mode_dropdown.add_css_class("magma-git-search-mode");
    mode_dropdown.set_selected(0);

    let search_entry = Entry::new();
    search_entry.add_css_class("magma-git-search-entry");
    search_entry.set_placeholder_text(Some("search commits..."));
    search_entry.set_hexpand(true);
    search_entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );

    search_row.append(&mode_dropdown);
    search_row.append(&search_entry);

    // Results
    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("magma-git-search-results");

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroller.set_child(Some(&list));

    let count_label = Label::new(None);
    count_label.add_css_class("magma-git-search-count");
    count_label.set_xalign(0.0);

    root.append(&search_row);
    root.append(&count_label);
    root.append(&scroller);

    let search_widgets = Rc::new(SearchWidgets {
        list: list.clone(),
        count_label: count_label.clone(),
    });

    *view.search_widgets.borrow_mut() = Some(search_widgets.clone());

    // Bind search on Enter
    {
        let view_ref = view.clone();
        let dropdown_ref = mode_dropdown.clone();
        let widgets = search_widgets.clone();
        search_entry.connect_activate(move |entry| {
            let query = entry.text().to_string();
            if query.is_empty() {
                return;
            }
            let mode = match dropdown_ref.selected() {
                0 => SearchMode::Message,
                1 => SearchMode::Author,
                2 => SearchMode::File,
                _ => SearchMode::Message,
            };
            perform_search(&view_ref, &widgets, &query, mode);
        });
    }

    root
}

fn perform_search(
    view: &Rc<GitPaneView>,
    widgets: &Rc<SearchWidgets>,
    query: &str,
    mode: SearchMode,
) {
    clear_list(&widgets.list);

    let root = view.repo_root.borrow().clone();
    let Some(root) = root else {
        return;
    };

    match ops::git_search_commits(&root, query, mode) {
        Ok(commits) => {
            widgets
                .count_label
                .set_text(&format!("{} results", commits.len()));

            if commits.is_empty() {
                let label = Label::new(Some("no matching commits"));
                label.add_css_class("magma-git-empty");
                label.set_xalign(0.0);
                widgets.list.append(&label);
                return;
            }

            for commit in &commits {
                widgets
                    .list
                    .append(&build_search_result(commit, view));
            }
        }
        Err(e) => {
            widgets.count_label.set_text("");
            let label = Label::new(Some(&format!("search error: {e}")));
            label.add_css_class("magma-git-error");
            widgets.list.append(&label);
        }
    }
}

fn build_search_result(commit: &CommitInfo, view: &Rc<GitPaneView>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("magma-git-commit-container");

    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-git-commit-row");

    let info_box = GtkBox::new(Orientation::Vertical, 1);
    info_box.set_hexpand(true);

    let top_row = GtkBox::new(Orientation::Horizontal, 6);
    let hash_label = Label::new(Some(&commit.short_hash));
    hash_label.add_css_class("magma-git-commit-hash");

    let msg_label = Label::new(Some(&commit.message));
    msg_label.add_css_class("magma-git-commit-msg");
    msg_label.set_xalign(0.0);
    msg_label.set_hexpand(true);
    msg_label.set_ellipsize(pango::EllipsizeMode::End);

    top_row.append(&hash_label);
    top_row.append(&msg_label);

    let bottom_row = GtkBox::new(Orientation::Horizontal, 8);
    let author_label = Label::new(Some(&commit.author));
    author_label.add_css_class("magma-git-commit-author");
    author_label.set_xalign(0.0);
    author_label.set_hexpand(true);

    let date_label = Label::new(Some(&commit.date));
    date_label.add_css_class("magma-git-commit-date");

    bottom_row.append(&author_label);
    bottom_row.append(&date_label);

    info_box.append(&top_row);
    info_box.append(&bottom_row);
    row.append(&info_box);

    // Expandable diff
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

            if let Ok(stat) = ops::git_diff_stat(&root, &commit_hash) {
                diff_box.append(&build_diff_stat(&stat));
            }

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

pub(super) struct SearchWidgets {
    pub(super) list: ListBox,
    pub(super) count_label: Label,
}
