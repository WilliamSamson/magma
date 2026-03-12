mod header;
mod input;
mod shell;
mod style;
mod terminal;

use std::io;

use gtk::{glib, prelude::*, Application, ApplicationWindow, Box as GtkBox, Orientation};
use vte4::prelude::*;
use winit::dpi::PhysicalSize;

use crate::window_state;

const APP_ID: &str = "io.obsidian";
const APP_TITLE: &str = "Obsidian";
const ICON_NAME: &str = "obsidian";
const HEADER_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon_64.png");
const MARGIN_HORIZONTAL: i32 = 16;
const MARGIN_TOP: i32 = 12;
const MARGIN_BOTTOM: i32 = 16;

pub(crate) fn run() -> io::Result<()> {
    let initial_size = window_state::load_window_size()?.unwrap_or_default();
    glib::set_application_name(APP_TITLE);
    glib::set_prgname(Some(APP_TITLE));
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| build_window(app, initial_size.width, initial_size.height));
    let _ = app.run();
    Ok(())
}

fn build_window(app: &Application, width: u32, height: u32) {
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    style::install_css();

    let header = header::build_header();
    let terminal = terminal::build_terminal();
    shell::spawn_shell(&terminal);

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("obsidian-shell");
    container.set_margin_start(MARGIN_HORIZONTAL);
    container.set_margin_end(MARGIN_HORIZONTAL);
    container.set_margin_top(MARGIN_TOP);
    container.set_margin_bottom(MARGIN_BOTTOM);
    container.append(&terminal);

    let entry = input::append_input_row(&container, &terminal);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .icon_name(ICON_NAME)
        .default_width(width.max(960) as i32)
        .default_height(height.max(620) as i32)
        .build();
    gtk::Window::set_default_icon_name(ICON_NAME);
    window.add_css_class("obsidian-window");
    window.set_titlebar(Some(&header));
    window.set_child(Some(&container));

    let app_for_exit = app.clone();
    terminal.connect_child_exited(move |_, _| {
        app_for_exit.quit();
    });

    window.connect_close_request(|window| {
        persist_window_size(window);
        glib::Propagation::Proceed
    });

    window.present();
    let _ = entry.grab_focus_without_selecting();
}

fn persist_window_size(window: &ApplicationWindow) {
    if window.is_maximized() {
        return;
    }

    let width = window.width().max(1) as u32;
    let height = window.height().max(1) as u32;
    if let Err(error) = window_state::save_window_size(PhysicalSize::new(width, height)) {
        eprintln!("window size save failed: {error}");
    }
}
