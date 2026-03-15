use std::rc::Rc;

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Entry, Label, ListBox, Orientation, PolicyType,
    ScrolledWindow, SelectionMode,
};

use super::{
    ops::{self, BranchInfo},
    GitPaneView,
};

pub(super) fn build_branches_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-branch-root");
    root.set_vexpand(true);

    // Create branch row
    let create_row = GtkBox::new(Orientation::Horizontal, 4);
    create_row.add_css_class("magma-git-branch-create");

    let create_entry = Entry::new();
    create_entry.add_css_class("magma-git-branch-entry");
    create_entry.set_placeholder_text(Some("new branch name..."));
    create_entry.set_hexpand(true);

    let create_btn = Button::builder()
        .label("create")
        .css_classes(["magma-git-action-btn"])
        .sensitive(false)
        .build();

    {
        let btn = create_btn.clone();
        create_entry.connect_changed(move |entry| {
            btn.set_sensitive(!entry.text().is_empty());
        });
    }

    {
        let view_ref = view.clone();
        let entry_ref = create_entry.clone();
        create_btn.connect_clicked(move |_| {
            let name = entry_ref.text().to_string();
            if name.is_empty() {
                return;
            }
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_create_branch(&root, &name) {
                    Ok(_) => {
                        entry_ref.set_text("");
                        view_ref.set_status_ok(&format!("created & switched to {name}"));
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("create failed: {e}")),
                }
            }
        });
    }

    // Also create on Enter
    {
        let btn = create_btn.clone();
        create_entry.connect_activate(move |_| {
            btn.emit_clicked();
        });
    }

    create_row.append(&create_entry);
    create_row.append(&create_btn);

    // Local branches
    let local_header = Label::new(Some("local"));
    local_header.add_css_class("magma-git-section-title");
    local_header.set_xalign(0.0);

    let local_list = ListBox::new();
    local_list.set_selection_mode(SelectionMode::None);
    local_list.add_css_class("magma-git-branch-list");

    // Remote branches
    let remote_header = Label::new(Some("remote"));
    remote_header.add_css_class("magma-git-section-title");
    remote_header.set_xalign(0.0);

    let remote_list = ListBox::new();
    remote_list.set_selection_mode(SelectionMode::None);
    remote_list.add_css_class("magma-git-branch-list");

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_policy(PolicyType::Never, PolicyType::Automatic);

    let inner = GtkBox::new(Orientation::Vertical, 4);
    inner.append(&local_header);
    inner.append(&local_list);
    inner.append(&remote_header);
    inner.append(&remote_list);
    scroller.set_child(Some(&inner));

    root.append(&create_row);
    root.append(&scroller);

    *view.branch_widgets.borrow_mut() = Some(BranchWidgets {
        local_list: local_list.clone(),
        remote_list: remote_list.clone(),
    });

    root
}

pub(super) fn refresh_branches(view: &Rc<GitPaneView>) {
    let widgets = view.branch_widgets.borrow();
    let Some(widgets) = widgets.as_ref() else {
        return;
    };

    clear_list(&widgets.local_list);
    clear_list(&widgets.remote_list);

    let root = view.repo_root.borrow().clone();
    let Some(root) = root else {
        return;
    };

    match ops::git_branches(&root) {
        Ok((local, remote)) => {
            for branch in &local {
                widgets.local_list.append(&build_branch_row(branch, view));
            }
            for branch in &remote {
                widgets.remote_list.append(&build_remote_branch_row(branch));
            }
        }
        Err(e) => {
            let label = Label::new(Some(&format!("error: {e}")));
            label.add_css_class("magma-git-error");
            widgets.local_list.append(&label);
        }
    }
}

fn build_branch_row(branch: &BranchInfo, view: &Rc<GitPaneView>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-git-branch-row");

    if branch.is_current {
        row.add_css_class("magma-git-branch-current");
    }

    let indicator = Label::new(Some(if branch.is_current { "\u{25CF}" } else { "" }));
    indicator.add_css_class("magma-git-branch-indicator");

    let name = Label::new(Some(&branch.name));
    name.add_css_class("magma-git-branch-name");
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(pango::EllipsizeMode::End);

    let commit = Label::new(Some(&branch.last_commit));
    commit.add_css_class("magma-git-branch-commit");
    commit.set_ellipsize(pango::EllipsizeMode::End);
    commit.set_max_width_chars(20);

    row.append(&indicator);
    row.append(&name);
    row.append(&commit);

    if !branch.is_current {
        // Switch button
        let switch_btn = Button::builder()
            .label("switch")
            .css_classes(["magma-git-file-action"])
            .build();

        let branch_name = branch.name.clone();
        let view_ref = view.clone();
        switch_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_switch_branch(&root, &branch_name) {
                    Ok(_) => {
                        view_ref.set_status_ok(&format!("switched to {branch_name}"));
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("switch failed: {e}")),
                }
            }
        });
        row.append(&switch_btn);

        // Delete button
        let delete_btn = Button::builder()
            .icon_name("edit-delete-symbolic")
            .css_classes(["magma-git-file-discard"])
            .tooltip_text("Delete branch")
            .build();

        let branch_name = branch.name.clone();
        let view_ref = view.clone();
        delete_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_delete_branch(&root, &branch_name) {
                    Ok(_) => {
                        view_ref.set_status_ok(&format!("deleted {branch_name}"));
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("delete failed: {e}")),
                }
            }
        });
        row.append(&delete_btn);

        // Merge button
        let merge_btn = Button::builder()
            .label("merge")
            .css_classes(["magma-git-file-action"])
            .tooltip_text("Merge into current branch")
            .build();

        let branch_name = branch.name.clone();
        let view_ref = view.clone();
        merge_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_merge_branch(&root, &branch_name) {
                    Ok(output) => {
                        let summary = output.lines().next().unwrap_or("merged");
                        view_ref.set_status_ok(summary);
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("merge failed: {e}")),
                }
            }
        });
        row.append(&merge_btn);
    }

    row
}

fn build_remote_branch_row(branch: &BranchInfo) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-git-branch-row");
    row.add_css_class("magma-git-branch-remote");

    let name = Label::new(Some(&branch.name));
    name.add_css_class("magma-git-branch-name");
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(pango::EllipsizeMode::End);

    let commit = Label::new(Some(&branch.last_commit));
    commit.add_css_class("magma-git-branch-commit");
    commit.set_ellipsize(pango::EllipsizeMode::End);
    commit.set_max_width_chars(25);

    row.append(&name);
    row.append(&commit);
    row
}

fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

pub(super) struct BranchWidgets {
    pub(super) local_list: ListBox,
    pub(super) remote_list: ListBox,
}
