use std::{cell::RefCell, rc::Rc};

use gtk::{Button, HeaderBar, Image, Label, MenuButton, glib, prelude::*};

use super::{APP_TITLE, HEADER_ICON_PATH};

type HeaderAction = Rc<dyn Fn()>;

#[derive(Clone)]
pub(super) struct AppHeader {
    header: HeaderBar,
    logo: Image,
    title: Label,
    settings_button: Button,
    inspector_button: MenuButton,
    close_button: Button,
    close_action: Rc<RefCell<Option<HeaderAction>>>,
}

pub(super) fn build_header() -> AppHeader {
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

    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["magma-header-close"])
        .tooltip_text("Close")
        .visible(false)
        .build();

    let settings_button = Button::builder()
        .icon_name("emblem-system-symbolic")
        .css_classes(["magma-header-settings"])
        .tooltip_text("Settings")
        .build();

    let inspector_button = MenuButton::new();
    inspector_button.add_css_class("magma-header-settings");
    inspector_button.set_icon_name("dialog-information-symbolic");
    inspector_button.set_tooltip_text(Some("Terminal inspector"));

    header.pack_start(&close_button);
    header.pack_end(&settings_button);
    header.pack_end(&inspector_button);

    let close_action: Rc<RefCell<Option<HeaderAction>>> = Rc::new(RefCell::new(None));
    let close_slot = close_action.clone();
    // Clone the action out of the cell before invoking it.
    // The action may trigger show_workspace_mode() which needs borrow_mut()
    // on the same RefCell, so the immutable borrow must be released first.
    close_button.connect_clicked(move |_| {
        let action = close_slot.borrow().clone();
        if let Some(action) = action {
            action();
        }
    });

    // Apply tooltips to window control buttons after GTK realizes the widget tree.
    let header_ref = header.clone();
    glib::idle_add_local_once(move || {
        apply_window_button_tooltips(&header_ref);
    });

    AppHeader {
        header,
        logo,
        title,
        settings_button,
        inspector_button,
        close_button,
        close_action,
    }
}

impl AppHeader {
    pub(super) fn widget(&self) -> &HeaderBar {
        &self.header
    }

    pub(super) fn settings_button(&self) -> &Button {
        &self.settings_button
    }

    pub(super) fn inspector_button(&self) -> &MenuButton {
        &self.inspector_button
    }

    pub(super) fn show_workspace_mode(&self, show_settings_button: bool, show_inspector: bool) {
        self.logo.set_visible(true);
        self.close_button.set_visible(false);
        self.settings_button.set_visible(show_settings_button);
        self.inspector_button.set_visible(show_inspector);
        self.title.set_text(APP_TITLE);
        self.title.remove_css_class("magma-settings-title");
        self.header.set_show_title_buttons(true);
        *self.close_action.borrow_mut() = None;
    }

    pub(super) fn show_settings_mode(&self) {
        self.logo.set_visible(true);
        self.close_button.set_visible(true);
        self.settings_button.set_visible(false);
        self.inspector_button.set_visible(false);
        self.title.remove_css_class("magma-settings-title");
        self.header.set_show_title_buttons(true);
    }

    pub(super) fn set_settings_title(&self, title: &str) {
        self.title.set_text(title);
    }

    pub(super) fn set_settings_close_action(&self, action: HeaderAction) {
        *self.close_action.borrow_mut() = Some(action);
    }
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
