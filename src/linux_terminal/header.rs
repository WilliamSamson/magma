use gtk::{prelude::*, HeaderBar, Image, Label};

use super::{APP_TITLE, HEADER_ICON_PATH};

pub(super) fn build_header() -> HeaderBar {
    let header = HeaderBar::new();
    header.add_css_class("obsidian-header");
    header.set_show_title_buttons(true);
    header.set_decoration_layout(Some(":minimize,maximize,close"));

    let logo = Image::from_file(HEADER_ICON_PATH);
    logo.set_pixel_size(16);
    logo.add_css_class("obsidian-logo");
    header.pack_start(&logo);

    let title = Label::new(Some(APP_TITLE));
    title.add_css_class("obsidian-title");
    header.set_title_widget(Some(&title));

    header
}
