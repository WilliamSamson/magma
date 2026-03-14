use std::{cell::RefCell, rc::Rc};

use gtk::{prelude::*, Box as GtkBox};

use super::{
    sections::preview_settings,
    widgets::{dropdown_row, section_label},
    Settings,
};

pub(super) fn build_browser_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("browser"));

    let browsers = ["google", "duckduckgo", "bing", "brave"];
    let selected = browsers
        .iter()
        .position(|browser| *browser == settings.borrow().default_browser)
        .unwrap_or(0) as u32;
    let browser_dropdown = dropdown_row(content, "default browser", &browsers, selected);
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
        preview_settings(&browser_settings, &browser_apply);
    });
}
