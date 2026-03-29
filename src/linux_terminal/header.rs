use std::{cell::RefCell, rc::Rc};

use gtk::{glib, prelude::*, Button, HeaderBar, Image, Label, MenuButton};

use super::{APP_TITLE, HEADER_ICON_PATH};

pub(super) fn build_header() -> (HeaderBar, Button, MenuButton) {
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

    let inspector_button = MenuButton::new();
    inspector_button.add_css_class("magma-header-settings");
    inspector_button.set_icon_name("dialog-information-symbolic");
    inspector_button.set_tooltip_text(Some("Terminal inspector"));

    header.pack_end(&settings_button);
    header.pack_end(&inspector_button);

    // Apply tooltips to window control buttons after GTK realizes the widget tree.
    let header_ref = header.clone();
    glib::idle_add_local_once(move || {
        apply_window_button_tooltips(&header_ref);
    });

    (header, settings_button, inspector_button)
}

/// Wire the inspector button to the active terminal provider.
/// Called after the workspace is available.
pub(super) fn wire_inspector(
    inspector_button: &MenuButton,
    terminal_provider: Rc<dyn Fn() -> Option<vte4::Terminal>>,
    settings: Rc<RefCell<super::settings::Settings>>,
) {
    use super::input::inspector;

    let popover = gtk::Popover::new();
    popover.set_has_arrow(false);
    popover.add_css_class("magma-inspector-popover");
    inspector_button.set_popover(Some(&popover));

    let terminal_provider = terminal_provider.clone();
    let settings = settings.clone();
    inspector_button.connect_notify_local(Some("active"), move |button, _| {
        if !button.property::<bool>("active") {
            return;
        }

        let popover = button.popover().unwrap();
        if let Some(terminal) = terminal_provider() {
            let content = inspector::build_inspector_panel(&terminal, &settings.borrow());
            popover.set_child(Some(&content));
        } else {
            let label = Label::new(Some("No active terminal"));
            label.add_css_class("magma-inspector-value");
            popover.set_child(Some(&label));
        }
    });
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
