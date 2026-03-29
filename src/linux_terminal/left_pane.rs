use std::{cell::Cell, rc::Rc};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, Image, Orientation, Overflow, PolicyType, Revealer,
    RevealerTransitionType, ScrolledWindow,
};

use super::{folder, folder::OnFileClick, view::CwdProvider};

const LEFT_PANE_WIDTH: i32 = 220;

#[derive(Clone)]
pub(super) struct LeftPane {
    handle: GtkBox,
    button: Button,
    revealer: Revealer,
    open: Rc<Cell<bool>>,
}

pub(super) fn build_left_pane(cwd_provider: CwdProvider, on_file_click: OnFileClick) -> LeftPane {
    let open = Rc::new(Cell::new(false));

    // Handle (single button strip on the left side)
    let handle = GtkBox::new(Orientation::Vertical, 3);
    handle.add_css_class("magma-left-handle");
    handle.add_css_class("collapsed");
    handle.set_vexpand(false);
    handle.set_valign(Align::Center);

    let folder_button = handle_button("folder-symbolic", "Explorer");
    handle.append(&folder_button);

    // Build the folder pane content
    let folder_pane = folder::build_folder_pane(cwd_provider, on_file_click);
    let wrapped = wrap_pane(&folder_pane);
    let revealer = build_revealer(&wrapped);

    let left_pane = LeftPane {
        handle: handle.clone(),
        button: folder_button.clone(),
        revealer,
        open,
    };

    {
        let left_pane = left_pane.clone();
        folder_button.connect_clicked(move |_| {
            left_pane.toggle();
        });
    }

    left_pane.sync();
    left_pane
}

impl LeftPane {
    pub(super) fn handle(&self) -> &GtkBox {
        &self.handle
    }

    pub(super) fn revealer(&self) -> &Revealer {
        &self.revealer
    }

    pub(super) fn toggle(&self) {
        self.open.set(!self.open.get());
        self.sync();
    }

    fn sync(&self) {
        let is_open = self.open.get();

        self.revealer.set_visible(is_open);
        self.revealer.set_reveal_child(is_open);

        if is_open {
            self.button.add_css_class("active");
            self.handle.remove_css_class("collapsed");
        } else {
            self.button.remove_css_class("active");
            self.handle.add_css_class("collapsed");
        }
    }
}

fn handle_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::builder()
        .css_classes(["magma-left-handle-segment"])
        .tooltip_text(tooltip)
        .build();
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("magma-left-handle-icon");
    button.set_child(Some(&icon));
    button
}

fn wrap_pane(child: &impl IsA<gtk::Widget>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-left-pane");
    root.set_size_request(LEFT_PANE_WIDTH, -1);
    root.set_width_request(LEFT_PANE_WIDTH);
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_valign(Align::Fill);
    root.set_overflow(Overflow::Hidden);
    root.append(child);
    root
}

fn build_revealer(child: &impl IsA<gtk::Widget>) -> Revealer {
    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .build();
    revealer.set_visible(false);
    revealer.set_hexpand(false);
    revealer.set_vexpand(true);
    revealer.set_halign(Align::Start);
    revealer.set_width_request(LEFT_PANE_WIDTH);

    let frame = ScrolledWindow::new();
    frame.set_hexpand(false);
    frame.set_vexpand(true);
    frame.set_min_content_width(LEFT_PANE_WIDTH);
    frame.set_max_content_width(LEFT_PANE_WIDTH);
    frame.set_propagate_natural_height(false);
    frame.set_propagate_natural_width(false);
    frame.set_policy(PolicyType::Never, PolicyType::Never);
    frame.set_child(Some(child));

    revealer.set_child(Some(&frame));
    revealer
}
