use std::{cell::RefCell, rc::Rc};

use gtk::{
    prelude::*,
    Box as GtkBox, Button, Label, Orientation, Stack, StackTransitionType,
};

use super::{about::build_about_page, sections::build_main_page, Settings};

pub(in crate::linux_terminal) fn build_settings_page(
    settings: Rc<RefCell<Settings>>,
    on_back: impl Fn() + 'static,
    on_apply: impl Fn(&Settings) + 'static,
    on_clear_browser_data: impl Fn() + 'static,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.add_css_class("magma-settings-root");

    let page_stack = Stack::new();
    page_stack.set_vexpand(true);
    page_stack.set_hexpand(true);
    page_stack.set_transition_type(StackTransitionType::Crossfade);
    page_stack.set_transition_duration(160);

    let title = Label::new(Some("settings"));
    let header = build_header(&page_stack, &title, Rc::new(on_back));
    bind_title_sync(&page_stack, &title);

    let on_apply: Rc<dyn Fn(&Settings)> = Rc::new(on_apply);
    let on_clear_browser_data: Rc<dyn Fn()> = Rc::new(on_clear_browser_data);
    let main_page = build_main_page(&page_stack, &settings, on_apply, on_clear_browser_data);
    let about_page = build_about_page();

    page_stack.add_named(&main_page, Some("main"));
    page_stack.add_named(&about_page, Some("about"));
    page_stack.set_visible_child_name("main");

    root.append(&header);
    root.append(&page_stack);
    root
}

fn build_header(page_stack: &Stack, title: &Label, on_back: Rc<dyn Fn()>) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.add_css_class("magma-settings-header");

    let back_button = Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["magma-settings-back"])
        .tooltip_text("Back")
        .build();

    let stack_ref = page_stack.clone();
    back_button.connect_clicked(move |_| {
        if stack_ref.visible_child_name().as_deref() == Some("about") {
            stack_ref.set_visible_child_name("main");
            return;
        }
        on_back();
    });

    title.add_css_class("magma-settings-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    header.append(&back_button);
    header.append(title);
    header
}

fn bind_title_sync(page_stack: &Stack, title: &Label) {
    let title_ref = title.clone();
    page_stack.connect_visible_child_name_notify(move |stack| {
        let next = if stack.visible_child_name().as_deref() == Some("about") {
            "about"
        } else {
            "settings"
        };
        title_ref.set_text(next);
    });
}
