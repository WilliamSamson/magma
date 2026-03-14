use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk, glib,
    prelude::*,
    Box as GtkBox, Button, Entry, EventControllerFocus, EventControllerKey, GestureClick,
    GestureDrag, Label, Notebook, Orientation, ScrolledWindow,
};

use super::{ops, super::tab::TabView};

const DRAG_THRESHOLD: f64 = 8.0;
pub(super) type RenameState = Rc<RefCell<Option<usize>>>;

/// Lightweight active-tab update: just toggles CSS classes without rebuilding widgets.
pub(super) fn update_active_tab(container: &GtkBox, notebook: &Notebook) {
    update_active_tab_at(container, current_index(notebook));
}

fn update_active_tab_at(container: &GtkBox, current: usize) {
    let mut index = 0usize;
    let mut child = container.first_child();
    while let Some(widget) = child {
        let is_active = index == current;

        if is_active {
            widget.add_css_class("active");
        } else {
            widget.remove_css_class("active");
        }

        update_children_active(&widget, is_active);

        index += 1;
        child = widget.next_sibling();
    }
}

pub(super) fn reveal_active_tab(
    container: &GtkBox,
    scroller: &ScrolledWindow,
    notebook: &Notebook,
) {
    reveal_active_tab_at(container, scroller, current_index(notebook));
}

pub(super) fn reveal_active_tab_at(
    container: &GtkBox,
    scroller: &ScrolledWindow,
    current: usize,
) {
    let container = container.clone();
    let scroller = scroller.clone();

    glib::idle_add_local_once(move || {
        let adjustment = scroller.hadjustment();

        let Some(widget) = nth_child(&container, current) else {
            return;
        };

        let allocation = widget.allocation();
        let left = allocation.x() as f64;
        let right = left + allocation.width() as f64;
        let current_value = adjustment.value();
        let visible_left = current_value;
        let visible_right = current_value + adjustment.page_size();

        let target = if left < visible_left {
            left
        } else if right > visible_right {
            right - adjustment.page_size()
        } else {
            current_value
        };

        let max_value = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
        adjustment.set_value(target.clamp(adjustment.lower(), max_value));
    });
}

/// Recursively update active classes on tab button labels and close buttons.
fn update_children_active(widget: &gtk::Widget, is_active: bool) {
    if widget.has_css_class("obsidian-tab-button-label")
        || widget.has_css_class("obsidian-tab-close-button")
    {
        if is_active {
            widget.add_css_class("active");
        } else {
            widget.remove_css_class("active");
        }
    }

    let mut child = widget.first_child();
    while let Some(c) = child {
        update_children_active(&c, is_active);
        child = c.next_sibling();
    }
}

/// Full rebuild of the tab strip. Used only when tabs are added or removed.
pub(super) fn rebuild_tab_strip(
    container: &GtkBox,
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    rename_state: &RenameState,
) {
    rebuild_tab_strip_at(container, notebook, tabs, rename_state, current_index(notebook));
}

pub(super) fn rebuild_tab_strip_at(
    container: &GtkBox,
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    rename_state: &RenameState,
    current: usize,
) {
    clear_children(container);
    let tab_count = tabs.borrow().len();

    for (index, tab) in tabs.borrow().iter().enumerate() {
        let is_active = index == current;
        let is_renaming = rename_state.borrow().is_some_and(|rename_index| rename_index == index);

        let tab_root = GtkBox::new(Orientation::Horizontal, 8);
        tab_root.add_css_class("obsidian-tab-item");
        if is_active {
            tab_root.add_css_class("active");
        }

        if is_renaming {
            let entry = rename_entry(tab.base_title(), index, tabs, notebook, container, rename_state);
            tab_root.append(&entry);
        } else {
            let title = tab.title_label().text();
            let label = Label::new(Some(&title));
            label.add_css_class("obsidian-tab-label");
            label.add_css_class("obsidian-tab-button-label");
            if is_active {
                label.add_css_class("active");
            }

            label.set_hexpand(true);
            tab_root.append(&label);
        }

        if tab_count > 1 {
            let close_button = Button::builder()
                .icon_name("window-close-symbolic")
                .css_classes(["obsidian-tab-close-button"])
                .focus_on_click(false)
                .build();
            if is_active {
                close_button.add_css_class("active");
            }

            let notebook_close = notebook.clone();
            let tabs_close = tabs.clone();
            let container_close = container.clone();
            let rename_state_close = rename_state.clone();
            let tab_root_widget = tab.root().clone();
            close_button.connect_clicked(move |_| {
                if let Some(page) = notebook_close.page_num(&tab_root_widget) {
                    let index = page as usize;
                    let _ = ops::close_tab_at(&tabs_close, &notebook_close, index);
                    rebuild_tab_strip(&container_close, &notebook_close, &tabs_close, &rename_state_close);
                }
            });
            tab_root.append(&close_button);
        }

        attach_rename(
            &tab_root,
            index,
            notebook,
            tabs,
            container,
            rename_state,
            is_renaming,
        );
        attach_drag(&tab_root, index, tabs, notebook, container, rename_state);

        container.append(&tab_root);
    }
}

pub(super) fn start_tab_rename(
    container: &GtkBox,
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    rename_state: &RenameState,
    index: usize,
) {
    if index >= tabs.borrow().len() {
        return;
    }

    *rename_state.borrow_mut() = Some(index);
    rebuild_tab_strip(container, notebook, tabs, rename_state);
}

fn activate_tab(notebook: &Notebook, index: usize) {
    notebook.set_current_page(Some(index as u32));

    let notebook = notebook.clone();
    glib::idle_add_local_once(move || {
        if let Some(page) = notebook.nth_page(Some(index as u32)) {
            let _ = page.child_focus(gtk::DirectionType::TabForward);
        }
    });
}

fn attach_drag(
    tab_root: &GtkBox,
    index: usize,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    container: &GtkBox,
    rename_state: &RenameState,
) {
    let gesture = GestureDrag::new();
    gesture.set_button(gdk::BUTTON_PRIMARY);

    // During drag: add visual feedback once past threshold
    {
        let tab_widget = tab_root.clone();
        let container_ref = container.clone();
        gesture.connect_drag_update(move |_gesture, offset_x, _offset_y| {
            if offset_x.abs() > DRAG_THRESHOLD {
                tab_widget.add_css_class("dragging");
                highlight_drop_target(&container_ref, &tab_widget, offset_x);
            }
        });
    }

    // On drag end: decide click vs reorder
    {
        let tab_widget = tab_root.clone();
        let container_ref = container.clone();
        let tabs_ref = tabs.clone();
        let notebook_ref = notebook.clone();
        let rename_state_ref = rename_state.clone();
        gesture.connect_drag_end(move |_gesture, offset_x, offset_y| {
            let was_dragging = tab_widget.has_css_class("dragging");
            tab_widget.remove_css_class("dragging");
            clear_drop_indicators(&container_ref);

            let distance = (offset_x * offset_x + offset_y * offset_y).sqrt();
            if distance >= DRAG_THRESHOLD && was_dragging {
                // Drag: find target and reorder
                if let Some(target) = find_target_index(&container_ref, &tab_widget, offset_x) {
                    if target != index {
                        ops::reorder_tab(&tabs_ref, &notebook_ref, index, target);
                        rebuild_tab_strip(
                            &container_ref,
                            &notebook_ref,
                            &tabs_ref,
                            &rename_state_ref,
                        );
                        notebook_ref.set_current_page(Some(target as u32));
                    }
                }
            } else {
                // Click: switch to this tab
                activate_tab(&notebook_ref, index);
            }
        });
    }

    tab_root.add_controller(gesture);
}

fn attach_rename(
    tab_root: &GtkBox,
    index: usize,
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    container: &GtkBox,
    rename_state: &RenameState,
    is_renaming: bool,
) {
    if is_renaming {
        return;
    }

    let click = GestureClick::new();
    click.set_button(gdk::BUTTON_PRIMARY);
    let notebook_ref = notebook.clone();
    let tabs_ref = tabs.clone();
    let container_ref = container.clone();
    let rename_state_ref = rename_state.clone();
    click.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press != 2 {
            return;
        }

        if notebook_ref.current_page() != Some(index as u32) {
            activate_tab(&notebook_ref, index);
        }
        start_tab_rename(
            &container_ref,
            &notebook_ref,
            &tabs_ref,
            &rename_state_ref,
            index,
        );
    });
    tab_root.add_controller(click);
}

fn rename_entry(
    title: &str,
    index: usize,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    container: &GtkBox,
    rename_state: &RenameState,
) -> Entry {
    let entry = Entry::new();
    entry.set_text(title);
    entry.set_width_chars(title.len().clamp(8, 24) as i32);
    entry.add_css_class("obsidian-tab-rename-entry");
    entry.set_hexpand(true);

    let tabs_activate = tabs.clone();
    let notebook_activate = notebook.clone();
    let container_activate = container.clone();
    let rename_state_activate = rename_state.clone();
    entry.connect_activate(move |entry| {
        finish_tab_rename(
            index,
            entry.text().as_str(),
            true,
            &tabs_activate,
            &notebook_activate,
            &container_activate,
            &rename_state_activate,
        );
    });

    let tabs_focus = tabs.clone();
    let notebook_focus = notebook.clone();
    let container_focus = container.clone();
    let rename_state_focus = rename_state.clone();
    let focus_controller = EventControllerFocus::new();
    focus_controller.connect_leave(move |controller| {
        let Some(widget) = controller.widget() else {
            return;
        };
        let Ok(entry) = widget.downcast::<Entry>() else {
            return;
        };
        finish_tab_rename(
            index,
            entry.text().as_str(),
            true,
            &tabs_focus,
            &notebook_focus,
            &container_focus,
            &rename_state_focus,
        );
    });
    entry.add_controller(focus_controller);

    let tabs_key = tabs.clone();
    let notebook_key = notebook.clone();
    let container_key = container.clone();
    let rename_state_key = rename_state.clone();
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |controller, key, _, _| {
        if key != gdk::Key::Escape {
            return glib::Propagation::Proceed;
        }

        let Some(widget) = controller.widget() else {
            return glib::Propagation::Proceed;
        };
        let Ok(entry) = widget.downcast::<Entry>() else {
            return glib::Propagation::Proceed;
        };
        finish_tab_rename(
            index,
            entry.text().as_str(),
            false,
            &tabs_key,
            &notebook_key,
            &container_key,
            &rename_state_key,
        );
        glib::Propagation::Stop
    });
    entry.add_controller(key_controller);

    let entry_ref = entry.clone();
    glib::idle_add_local_once(move || {
        entry_ref.grab_focus();
        entry_ref.select_region(0, -1);
    });

    entry
}

fn finish_tab_rename(
    index: usize,
    title: &str,
    commit: bool,
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    container: &GtkBox,
    rename_state: &RenameState,
) {
    if rename_state.borrow().as_ref() != Some(&index) {
        return;
    }

    let normalized_title = title.trim().to_string();

    if commit && !normalized_title.is_empty() {
        if let Some(tab) = tabs.borrow_mut().get_mut(index) {
            tab.rename(&normalized_title);
        }
    }

    *rename_state.borrow_mut() = None;

    let container = container.clone();
    let notebook = notebook.clone();
    let tabs = tabs.clone();
    let rename_state = rename_state.clone();
    glib::idle_add_local_once(move || {
        rebuild_tab_strip(&container, &notebook, &tabs, &rename_state);
    });
}

/// Highlight the tab under the drag cursor with `drop-target` CSS class.
fn highlight_drop_target(container: &GtkBox, source_widget: &GtkBox, offset_x: f64) {
    let source_alloc = source_widget.allocation();
    let cursor_x = source_alloc.x() as f64 + source_alloc.width() as f64 / 2.0 + offset_x;

    let mut child = container.first_child();
    while let Some(widget) = child {
        let alloc = widget.allocation();
        let left = alloc.x() as f64;
        let right = left + alloc.width() as f64;
        let is_source = widget.eq(source_widget.upcast_ref::<gtk::Widget>());

        if !is_source && cursor_x >= left && cursor_x < right {
            widget.add_css_class("drop-target");
        } else {
            widget.remove_css_class("drop-target");
        }

        child = widget.next_sibling();
    }
}

/// Find which tab index the cursor is over based on drag offset.
fn find_target_index(container: &GtkBox, source_widget: &GtkBox, offset_x: f64) -> Option<usize> {
    let source_alloc = source_widget.allocation();
    let cursor_x = source_alloc.x() as f64 + source_alloc.width() as f64 / 2.0 + offset_x;

    let mut child_idx = 0usize;
    let mut child = container.first_child();
    while let Some(widget) = child {
        let alloc = widget.allocation();
        let left = alloc.x() as f64;
        let right = left + alloc.width() as f64;

        if cursor_x >= left && cursor_x < right {
            return Some(child_idx);
        }

        child_idx += 1;
        child = widget.next_sibling();
    }

    // If cursor is past the last tab, target the last position
    if child_idx > 0 {
        Some(child_idx - 1)
    } else {
        None
    }
}

fn clear_drop_indicators(container: &GtkBox) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        widget.remove_css_class("drop-target");
        child = widget.next_sibling();
    }
}

fn clear_children(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn nth_child(container: &GtkBox, index: usize) -> Option<gtk::Widget> {
    let mut current = 0usize;
    let mut child = container.first_child();
    while let Some(widget) = child {
        if current == index {
            return Some(widget);
        }

        current += 1;
        child = widget.next_sibling();
    }

    None
}

fn current_index(notebook: &Notebook) -> usize {
    notebook.current_page().map(|index| index as usize).unwrap_or(0)
}
