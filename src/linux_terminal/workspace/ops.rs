use std::{cell::RefCell, rc::Rc};

use gtk::Notebook;

use super::super::terminal::TabView;

pub(super) fn close_tab_at(
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    index: usize,
) -> bool {
    if tabs.borrow().len() <= 1 || index >= tabs.borrow().len() {
        return false;
    }

    let next_index = index.saturating_sub(1);
    tabs.borrow_mut().remove(index);
    notebook.remove_page(Some(index as u32));

    let remaining = tabs.borrow().len();
    if remaining > 0 {
        notebook.set_current_page(Some(next_index.min(remaining.saturating_sub(1)) as u32));
    }

    true
}

pub(super) fn reorder_tab(
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    from: usize,
    to: usize,
) {
    let len = tabs.borrow().len();
    if from == to || from >= len || to >= len {
        return;
    }

    // Reorder the notebook page
    if let Some(page_widget) = notebook.nth_page(Some(from as u32)) {
        notebook.reorder_child(&page_widget, Some(to as u32));
    }

    // Reorder the tabs vec to match
    let tab = tabs.borrow_mut().remove(from);
    tabs.borrow_mut().insert(to, tab);
}
