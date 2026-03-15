use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk, glib,
    prelude::*,
    Box as GtkBox, Entry, EventControllerKey, Label, ListBox, ListBoxRow, Notebook, Orientation,
    ScrolledWindow,
};

use super::super::tab::TabView;

#[derive(Clone)]
pub(super) struct QuickSwitcher {
    root: GtkBox,
    entry: Entry,
    list: ListBox,
    notebook: Notebook,
    tabs: Rc<RefCell<Vec<TabView>>>,
    visible_tabs: Rc<RefCell<Vec<usize>>>,
}

impl QuickSwitcher {
    pub(super) fn new(notebook: &Notebook, tabs: &Rc<RefCell<Vec<TabView>>>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 10);
        root.add_css_class("magma-switcher-overlay");
        root.set_halign(gtk::Align::Center);
        root.set_valign(gtk::Align::Start);
        root.set_margin_top(56);
        root.set_visible(false);

        let panel = GtkBox::new(Orientation::Vertical, 8);
        panel.add_css_class("magma-switcher-panel");

        let entry = Entry::new();
        entry.add_css_class("magma-switcher-entry");
        entry.set_placeholder_text(Some("switch tab..."));

        let list = ListBox::new();
        list.add_css_class("magma-switcher-list");
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.set_activate_on_single_click(true);

        let scroller = ScrolledWindow::new();
        scroller.set_min_content_width(360);
        scroller.set_min_content_height(72);
        scroller.set_max_content_height(280);
        scroller.set_propagate_natural_height(true);
        scroller.set_child(Some(&list));

        panel.append(&entry);
        panel.append(&scroller);
        root.append(&panel);

        let switcher = Self {
            root,
            entry,
            list,
            notebook: notebook.clone(),
            tabs: tabs.clone(),
            visible_tabs: Rc::new(RefCell::new(Vec::new())),
        };

        switcher.bind_events();
        switcher
    }

    pub(super) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn is_open(&self) -> bool {
        self.root.is_visible()
    }

    pub(super) fn toggle(&self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }

    pub(super) fn open(&self) {
        self.root.set_visible(true);
        self.entry.set_text("");
        self.refresh();
        self.entry.grab_focus();
    }

    pub(super) fn close(&self) {
        self.root.set_visible(false);
        self.entry.set_text("");

        let notebook = self.notebook.clone();
        glib::idle_add_local_once(move || {
            if let Some(page) = notebook.nth_page(notebook.current_page()) {
                let _ = page.child_focus(gtk::DirectionType::TabForward);
            }
        });
    }

    pub(super) fn refresh(&self) {
        if !self.is_open() {
            return;
        }
        self.populate(self.entry.text().as_str());
    }

    fn bind_events(&self) {
        let switcher = self.clone();
        self.entry.connect_changed(move |entry| {
            switcher.populate(entry.text().as_str());
        });

        let switcher = self.clone();
        self.list.connect_row_activated(move |_, row| {
            switcher.activate_row(row.index() as usize);
        });

        let switcher = self.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| match key {
            gdk::Key::Escape => {
                switcher.close();
                glib::Propagation::Stop
            }
            gdk::Key::Down => {
                switcher.move_selection(1);
                glib::Propagation::Stop
            }
            gdk::Key::Up => {
                switcher.move_selection(-1);
                glib::Propagation::Stop
            }
            gdk::Key::Return => {
                if let Some(row) = switcher.list.selected_row() {
                    switcher.activate_row(row.index() as usize);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
            _ => glib::Propagation::Proceed,
        });
        self.entry.add_controller(key_controller);
    }

    fn populate(&self, query: &str) {
        clear_rows(&self.list);
        self.visible_tabs.borrow_mut().clear();

        let query = query.trim().to_lowercase();
        for (index, tab) in self.tabs.borrow().iter().enumerate() {
            let title = tab.title_label().text().to_string();
            if !query.is_empty() && !title.to_lowercase().contains(&query) {
                continue;
            }

            self.visible_tabs.borrow_mut().push(index);
            self.list.append(&switcher_row(index, &title));
        }

        if self.visible_tabs.borrow().is_empty() {
            self.list.append(&empty_row());
            return;
        }

        let selected = self
            .visible_tabs
            .borrow()
            .iter()
            .position(|index| *index == current_index(&self.notebook))
            .unwrap_or(0);
        if let Some(row) = self.list.row_at_index(selected as i32) {
            self.list.select_row(Some(&row));
        }
    }

    fn move_selection(&self, delta: i32) {
        if self.visible_tabs.borrow().is_empty() {
            return;
        }

        let current = self.list.selected_row().map(|row| row.index()).unwrap_or(0);
        let next = (current + delta).clamp(0, self.visible_tabs.borrow().len() as i32 - 1);
        if let Some(row) = self.list.row_at_index(next) {
            self.list.select_row(Some(&row));
            row.grab_focus();
        }
    }

    fn activate_row(&self, row_index: usize) {
        let Some(tab_index) = self.visible_tabs.borrow().get(row_index).copied() else {
            return;
        };

        self.notebook.set_current_page(Some(tab_index as u32));
        self.close();
    }
}

fn switcher_row(index: usize, title: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("magma-switcher-row");

    let content = GtkBox::new(Orientation::Horizontal, 10);

    let index_label = Label::new(Some(&(index + 1).to_string()));
    index_label.add_css_class("magma-switcher-index");

    let title_label = Label::new(Some(title));
    title_label.add_css_class("magma-switcher-label");
    title_label.set_xalign(0.0);

    content.append(&index_label);
    content.append(&title_label);
    row.set_child(Some(&content));
    row
}

fn empty_row() -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);

    let label = Label::new(Some("no matching tabs"));
    label.add_css_class("magma-switcher-empty");
    label.set_xalign(0.0);
    row.set_child(Some(&label));
    row
}

fn clear_rows(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn current_index(notebook: &Notebook) -> usize {
    notebook.current_page().map(|index| index as usize).unwrap_or(0)
}
