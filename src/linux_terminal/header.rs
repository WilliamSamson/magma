use gtk::{glib, prelude::*, Button, HeaderBar, Image, Label};

use super::{APP_TITLE, HEADER_ICON_PATH};

pub(super) fn build_header() -> (HeaderBar, Button) {
    let header = HeaderBar::new();
    header.add_css_class("magma-header");
    header.set_show_title_buttons(true);
    header.set_decoration_layout(Some(":close,minimize,maximize"));

    let logo = Image::from_file(HEADER_ICON_PATH);
    logo.set_pixel_size(16);
    logo.add_css_class("magma-logo");
    header.pack_start(&logo);

    let title = Label::new(Some(APP_TITLE));
    title.add_css_class("magma-title");
    header.set_title_widget(Some(&title));

    let settings_button = Button::builder()
        .icon_name("emblem-system-symbolic")
        .css_classes(["magma-header-settings"])
        .tooltip_text("Settings")
        .build();
    header.pack_end(&settings_button);

    // Apply tooltips to window control buttons after GTK realizes the widget tree.
    let header_ref = header.clone();
    glib::idle_add_local_once(move || {
        apply_window_button_tooltips(&header_ref);
    });

    (header, settings_button)
}

fn apply_window_button_tooltips(widget: &impl IsA<gtk::Widget>) {
    let mut child = widget.as_ref().first_child();
    while let Some(ref current) = child {
        if let Ok(button) = current.clone().downcast::<Button>() {
            if button.has_css_class("close") {
                button.set_tooltip_text(Some("Close"));
            } else if button.has_css_class("minimize") {
                button.set_tooltip_text(Some("Minimize"));
            } else if button.has_css_class("maximize") {
                button.set_tooltip_text(Some("Maximize"));
            }
        }
        apply_window_button_tooltips(current);
        child = current.next_sibling();
    }
}
