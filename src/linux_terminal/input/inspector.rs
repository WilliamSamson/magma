use std::{cell::RefCell, os::fd::AsRawFd, rc::Rc};

use gtk::{
    gio, glib, prelude::*, Box as GtkBox, Label, MenuButton, Orientation, Popover,
};
use vte4::{prelude::*, Terminal};

use crate::linux_terminal::settings::Settings;

#[derive(Clone)]
struct InspectorLabels {
    cwd: Label,
    file: Label,
    title: Label,
    grid: Label,
    font: Label,
    pty: Label,
    selection: Label,
    image: Label,
    ligatures: Label,
}

pub(super) fn build_inspector_button(
    terminal: &Terminal,
    settings: Rc<RefCell<Settings>>,
) -> MenuButton {
    let popover = Popover::new();
    popover.set_has_arrow(false);
    popover.add_css_class("magma-inspector-popover");

    let content = GtkBox::new(Orientation::Vertical, 10);
    content.add_css_class("magma-inspector-panel");

    let title = Label::new(Some("terminal inspector"));
    title.add_css_class("magma-inspector-title");
    title.set_xalign(0.0);
    content.append(&title);

    let labels = InspectorLabels {
        cwd: inspector_row(&content, "cwd"),
        file: inspector_row(&content, "file"),
        title: inspector_row(&content, "title"),
        grid: inspector_row(&content, "grid"),
        font: inspector_row(&content, "font"),
        pty: inspector_row(&content, "pty"),
        selection: inspector_row(&content, "selection"),
        image: inspector_row(&content, "image rendering"),
        ligatures: inspector_row(&content, "ligatures"),
    };

    popover.set_child(Some(&content));

    let button = MenuButton::new();
    button.add_css_class("magma-tool-menu");
    button.set_icon_name("dialog-information-symbolic");
    button.set_tooltip_text(Some("Terminal inspector"));
    button.set_popover(Some(&popover));

    let refresh: Rc<dyn Fn()> = {
        let terminal = terminal.clone();
        let labels = labels.clone();
        let settings = settings.clone();
        Rc::new(move || refresh_inspector(&terminal, &labels, &settings))
    };

    refresh();

    {
        let refresh = refresh.clone();
        button.connect_notify_local(Some("active"), move |button, _| {
            if button.property::<bool>("active") {
                refresh();
            }
        });
    }

    bind_terminal_refresh(terminal, &refresh);

    button
}

fn bind_terminal_refresh(terminal: &Terminal, refresh: &Rc<dyn Fn()>) {
    {
        let refresh = refresh.clone();
        terminal.connect_current_directory_uri_changed(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_current_file_uri_changed(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_window_title_changed(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_selection_changed(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_resize_window(move |_, _, _| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_font_desc_notify(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_font_scale_notify(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_pty_notify(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_enable_sixel_notify(move |_| refresh());
    }
    {
        let refresh = refresh.clone();
        terminal.connect_enable_shaping_notify(move |_| refresh());
    }
}

fn refresh_inspector(
    terminal: &Terminal,
    labels: &InspectorLabels,
    settings: &Rc<RefCell<Settings>>,
) {
    let settings = settings.borrow();
    labels.cwd.set_text(&display_uri(terminal.current_directory_uri()));
    labels.file.set_text(&display_uri(terminal.current_file_uri()));
    labels.title.set_text(
        terminal
            .window_title()
            .as_deref()
            .filter(|title| !title.is_empty())
            .unwrap_or("none"),
    );
    labels
        .grid
        .set_text(&format!("{} cols × {} rows", terminal.column_count(), terminal.row_count()));
    labels.font.set_text(&display_font(terminal));
    labels.pty.set_text(&display_pty(terminal));
    labels
        .selection
        .set_text(if terminal.has_selection() { "active" } else { "none" });
    labels
        .image
        .set_text(if settings.image_rendering { "enabled" } else { "disabled" });
    labels.ligatures.set_text(if settings.ligatures {
        "enabled"
    } else {
        "disabled"
    });
}

fn inspector_row(content: &GtkBox, key: &str) -> Label {
    let row = GtkBox::new(Orientation::Vertical, 3);
    row.add_css_class("magma-inspector-row");

    let key_label = Label::new(Some(key));
    key_label.add_css_class("magma-inspector-key");
    key_label.set_xalign(0.0);

    let value = Label::new(None);
    value.add_css_class("magma-inspector-value");
    value.set_xalign(0.0);
    value.set_wrap(true);
    value.set_selectable(false);

    row.append(&key_label);
    row.append(&value);
    content.append(&row);
    value
}

fn display_uri(uri: Option<glib::GString>) -> String {
    uri.as_deref()
        .and_then(|uri| gio::File::for_uri(uri).path())
        .map(|path| path.display().to_string())
        .or_else(|| uri.map(|value| value.to_string()))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_string())
}

fn display_font(terminal: &Terminal) -> String {
    let desc = terminal
        .font_desc()
        .map(|font| font.to_string())
        .unwrap_or_else(|| "default".to_string());
    format!("{desc} · scale {:.2}", terminal.font_scale())
}

fn display_pty(terminal: &Terminal) -> String {
    terminal
        .pty()
        .map(|pty| format!("attached · fd {}", pty.fd().as_raw_fd()))
        .unwrap_or_else(|| "detached".to_string())
}
