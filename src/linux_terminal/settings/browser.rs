use std::{cell::RefCell, rc::Rc};

use gtk::{prelude::*, Box as GtkBox};

use super::{
    save_settings,
    widgets::{action_row, dropdown_row, section_label},
    Settings,
};

pub(super) fn build_browser_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
    on_clear_browser_data: &Rc<dyn Fn()>,
) {
    content.append(&section_label("browser"));

    let browsers = ["google", "duckduckgo", "bing", "brave"];
    let selected = browsers
        .iter()
        .position(|browser| *browser == settings.borrow().default_browser)
        .unwrap_or(0) as u32;
    let browser_dropdown = dropdown_row(
        content,
        "default search engine",
        "the engine used by the web pane to resolve search queries.",
        &browsers,
        selected,
    );
    let browser_settings = settings.clone();
    let browser_apply = on_apply.clone();
    browser_dropdown.connect_selected_notify(move |dropdown| {
        let browser = match dropdown.selected() {
            1 => "duckduckgo",
            2 => "bing",
            3 => "brave",
            _ => "google",
        };
        browser_settings.borrow_mut().default_browser = browser.to_string();
        let snapshot = browser_settings.borrow().clone();
        save_settings(&snapshot);
        browser_apply(&snapshot);
    });

    let clear_button = action_row(
        content,
        "clear browser data",
        "remove saved sign-ins, cookies, site data, and restored browser tabs.",
        "clear",
    );
    let clear_browser_data = on_clear_browser_data.clone();
    clear_button.connect_clicked(move |_| clear_browser_data());
}
