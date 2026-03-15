use std::rc::Rc;

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Entry, Label, ListBox, Orientation, PolicyType,
    Revealer, RevealerTransitionType, ScrolledWindow, SelectionMode,
};

use super::{
    diff::build_diff_widget,
    ops::{self, StashEntry},
    GitPaneView,
};

pub(super) fn build_stash_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-stash-root");
    root.set_vexpand(true);

    // Stash push row
    let push_row = GtkBox::new(Orientation::Horizontal, 4);
    push_row.add_css_class("magma-git-stash-push");

    let stash_entry = Entry::new();
    stash_entry.add_css_class("magma-git-stash-entry");
    stash_entry.set_placeholder_text(Some("stash message (optional)"));
    stash_entry.set_hexpand(true);

    let stash_btn = Button::builder()
        .label("stash")
        .css_classes(["magma-git-action-btn"])
        .build();

    {
        let view_ref = view.clone();
        let entry_ref = stash_entry.clone();
        stash_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                let msg = entry_ref.text().to_string();
                let msg_opt = if msg.is_empty() { None } else { Some(msg.as_str()) };
                match ops::git_stash_push(&root, msg_opt) {
                    Ok(_) => {
                        entry_ref.set_text("");
                        view_ref.set_status_ok("stashed changes");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("stash failed: {e}")),
                }
            }
        });
    }

    push_row.append(&stash_entry);
    push_row.append(&stash_btn);

    // Stash list
    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("magma-git-stash-list");

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroller.set_child(Some(&list));

    root.append(&push_row);
    root.append(&scroller);

    *view.stash_widgets.borrow_mut() = Some(StashWidgets {
        list: list.clone(),
    });

    root
}

pub(super) fn refresh_stash(view: &Rc<GitPaneView>) {
    let widgets = view.stash_widgets.borrow();
    let Some(widgets) = widgets.as_ref() else {
        return;
    };

    clear_list(&widgets.list);

    let root = view.repo_root.borrow().clone();
    let Some(root) = root else {
        return;
    };

    match ops::git_stash_list(&root) {
        Ok(entries) => {
            if entries.is_empty() {
                let label = Label::new(Some("no stashes"));
                label.add_css_class("magma-git-empty");
                label.set_xalign(0.0);
                widgets.list.append(&label);
                return;
            }
            for entry in &entries {
                widgets.list.append(&build_stash_row(entry, view));
            }
        }
        Err(e) => {
            let label = Label::new(Some(&format!("error: {e}")));
            label.add_css_class("magma-git-error");
            widgets.list.append(&label);
        }
    }
}

fn build_stash_row(entry: &StashEntry, view: &Rc<GitPaneView>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("magma-git-stash-row-container");

    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-git-stash-row");

    let index_label = Label::new(Some(&format!("{{{}}}", entry.index)));
    index_label.add_css_class("magma-git-stash-index");

    let msg_label = Label::new(Some(&entry.message));
    msg_label.add_css_class("magma-git-stash-msg");
    msg_label.set_xalign(0.0);
    msg_label.set_hexpand(true);
    msg_label.set_ellipsize(pango::EllipsizeMode::End);

    let apply_btn = Button::builder()
        .label("apply")
        .css_classes(["magma-git-file-action"])
        .tooltip_text("Apply stash (keep in list)")
        .build();

    let pop_btn = Button::builder()
        .label("pop")
        .css_classes(["magma-git-file-action"])
        .tooltip_text("Apply and remove from list")
        .build();

    let drop_btn = Button::builder()
        .icon_name("edit-delete-symbolic")
        .css_classes(["magma-git-file-discard"])
        .tooltip_text("Drop stash")
        .build();

    let idx = entry.index;

    {
        let view_ref = view.clone();
        apply_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_stash_apply(&root, idx) {
                    Ok(_) => {
                        view_ref.set_status_ok("stash applied");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("apply failed: {e}")),
                }
            }
        });
    }

    {
        let view_ref = view.clone();
        pop_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_stash_pop(&root, idx) {
                    Ok(_) => {
                        view_ref.set_status_ok("stash popped");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("pop failed: {e}")),
                }
            }
        });
    }

    {
        let view_ref = view.clone();
        drop_btn.connect_clicked(move |_| {
            let root = view_ref.repo_root.borrow().clone();
            if let Some(root) = root {
                match ops::git_stash_drop(&root, idx) {
                    Ok(_) => {
                        view_ref.set_status_ok("stash dropped");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status_err(&format!("drop failed: {e}")),
                }
            }
        });
    }

    row.append(&index_label);
    row.append(&msg_label);
    row.append(&apply_btn);
    row.append(&pop_btn);
    row.append(&drop_btn);

    // Expandable diff preview on click
    let diff_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(200)
        .build();

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
            match ops::git_stash_show(&root, idx) {
                Ok(diff_text) => {
                    let hunks = ops::parse_diff_text(&diff_text);
                    let diff_widget = build_diff_widget(&hunks, None);
                    revealer_ref.set_child(Some(&diff_widget));
                    revealer_ref.set_reveal_child(true);
                }
                Err(e) => {
                    view_ref.set_status_err(&format!("stash preview failed: {e}"));
                }
            }
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

pub(super) struct StashWidgets {
    pub(super) list: ListBox,
}
