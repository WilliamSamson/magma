use std::{cell::RefCell, rc::Rc};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, Orientation, Overflow, Stack, StackTransitionType,
};

use super::{logr, settings::Settings, web};

pub(super) fn build_right_pane(settings: Rc<RefCell<Settings>>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-right-pane");
    root.set_size_request(420, -1);
    root.set_width_request(420);
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_valign(Align::Fill);
    root.set_overflow(Overflow::Hidden);

    let switcher = GtkBox::new(Orientation::Horizontal, 6);
    switcher.add_css_class("obsidian-sidepane-switcher");

    let logr_button = pane_button("logr", true);
    let web_button = pane_button("web", false);
    switcher.append(&logr_button);
    switcher.append(&web_button);
    root.append(&switcher);

    let stack = Stack::new();
    stack.add_css_class("obsidian-sidepane-stack");
    stack.set_hexpand(false);
    stack.set_vexpand(true);
    stack.set_width_request(0);
    stack.set_overflow(Overflow::Hidden);
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(160);
    stack.add_named(&logr::build_logr_pane(), Some("logr"));
    stack.add_named(&web::build_web_pane(settings), Some("web"));
    stack.set_visible_child_name("logr");
    root.append(&stack);

    {
        let stack = stack.clone();
        let logr_button_ref = logr_button.clone();
        let web_button_ref = web_button.clone();
        logr_button.connect_clicked(move |_| {
            stack.set_visible_child_name("logr");
            set_active_button(&logr_button_ref, &web_button_ref);
        });
    }

    {
        let stack = stack.clone();
        let logr_button_ref = logr_button.clone();
        let web_button_ref = web_button.clone();
        web_button.connect_clicked(move |_| {
            stack.set_visible_child_name("web");
            set_active_button(&web_button_ref, &logr_button_ref);
        });
    }

    root
}

fn pane_button(label: &str, active: bool) -> Button {
    let button = Button::builder()
        .label(label)
        .css_classes(["obsidian-sidepane-button"])
        .build();
    if active {
        button.add_css_class("active");
    }
    button
}

fn set_active_button(active: &Button, inactive: &Button) {
    active.add_css_class("active");
    inactive.remove_css_class("active");
}
