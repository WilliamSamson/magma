mod header;
mod input;
mod logr;
mod meta;
mod persist;
mod profile;
mod right_pane;
mod session;
mod settings;
mod shell;
mod style;
mod tab;
mod terminal;
mod web;
mod workspace;

use std::io;

use gtk::{
    gdk, gio, glib, prelude::*, Align, Application, ApplicationWindow, Box as GtkBox,
    GestureClick, IconTheme, Label, Orientation, PolicyType, Revealer,
    RevealerTransitionType, ScrolledWindow, Stack, StackTransitionType,
};
use std::{cell::RefCell, rc::Rc};
use winit::dpi::PhysicalSize;

use crate::window_state;

const APP_ID: &str = "io.obsidian.terminal";
const APP_TITLE: &str = "Obsidian";
const HEADER_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon_64.png");
const ICON_THEME_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
const MARGIN_HORIZONTAL: i32 = 16;
const MARGIN_TOP: i32 = 4;
const MARGIN_BOTTOM: i32 = 16;

pub(crate) fn run() -> io::Result<()> {
    let initial_size = window_state::load_window_size()?.unwrap_or_default();
    if let Err(error) = glib::setenv("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1", true) {
        eprintln!("webkit sandbox override failed: {error}");
    }
    glib::set_application_name(APP_TITLE);
    glib::set_prgname(Some(APP_ID));
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| build_window(app, initial_size.width, initial_size.height));
    let _ = app.run();
    Ok(())
}

fn build_window(app: &Application, width: u32, height: u32) {
    // Rc<RefCell<Settings>> shares the mutable runtime settings across settings UI and workspace callbacks on the GTK thread.
    let app_settings = Rc::new(RefCell::new(settings::load_settings()));

    if let Some(gtk_settings) = gtk::Settings::default() {
        gtk_settings.set_gtk_application_prefer_dark_theme(true);
    }

    // Register bundled icon theme so the taskbar/desktop can find the app icon
    if let Some(display) = gdk::Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        icon_theme.add_search_path(ICON_THEME_PATH);
    }

    style::install_css(app_settings.borrow().app_font_size);

    let (header, settings_button) = header::build_header();
    let workspace = std::rc::Rc::new(workspace::WorkspaceView::new(app_settings.clone()));
    let container = shell_container(
        workspace.root(),
        app_settings.borrow().logr_panel_open,
        app_settings.clone(),
    );

    // Stack: workspace (main) <-> settings
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    stack.add_named(&container, Some("workspace"));

    let stack_ref = stack.clone();
    let workspace_ref = workspace.clone();
    let settings_page = settings::build_settings_page(
        move || {
            stack_ref.set_visible_child_name("workspace");
        },
        move |new_settings| {
            *app_settings.borrow_mut() = new_settings.clone();
            style::install_css(new_settings.app_font_size);
            workspace_ref.apply_settings(new_settings);
        },
    );
    stack.add_named(&settings_page, Some("settings"));

    // Settings button toggles to settings view
    {
        let stack_ref = stack.clone();
        settings_button.connect_clicked(move |_| {
            let current = stack_ref.visible_child_name();
            if current.as_deref() == Some("settings") {
                stack_ref.set_visible_child_name("workspace");
            } else {
                stack_ref.set_visible_child_name("settings");
            }
        });
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .icon_name(APP_ID)
        .default_width(width.max(960) as i32)
        .default_height(height.max(620) as i32)
        .build();
    gtk::Window::set_default_icon_name(APP_ID);
    window.add_css_class("obsidian-window");
    window.set_titlebar(Some(&header));
    window.set_child(Some(&stack));

    window.connect_close_request(move |window| {
        workspace.save();
        persist_window_size(window);
        glib::Propagation::Proceed
    });

    window.present();
}

fn shell_container(
    child: &impl IsA<gtk::Widget>,
    logr_panel_open: bool,
    settings: Rc<RefCell<settings::Settings>>,
) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("obsidian-shell");
    container.set_margin_start(MARGIN_HORIZONTAL);
    container.set_margin_end(MARGIN_HORIZONTAL);
    container.set_margin_top(MARGIN_TOP);
    container.set_margin_bottom(MARGIN_BOTTOM);

    let view_row = GtkBox::new(Orientation::Horizontal, 0);
    view_row.set_spacing(2);
    view_row.set_vexpand(true);
    view_row.append(child);

    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideLeft)
        .transition_duration(250)
        .build();
    revealer.set_hexpand(false);
    revealer.set_vexpand(true);
    revealer.set_halign(Align::End);
    revealer.set_width_request(420);

    let right_pane = right_pane::build_right_pane(settings);
    let pane_frame = ScrolledWindow::new();
    pane_frame.set_hexpand(false);
    pane_frame.set_vexpand(true);
    pane_frame.set_min_content_width(420);
    pane_frame.set_max_content_width(420);
    pane_frame.set_propagate_natural_width(false);
    pane_frame.set_policy(PolicyType::Never, PolicyType::Never);
    pane_frame.set_child(Some(&right_pane));

    revealer.set_child(Some(&pane_frame));
    revealer.set_reveal_child(logr_panel_open);

    let handle = GtkBox::new(Orientation::Vertical, 0);
    handle.add_css_class("obsidian-handle");
    if !logr_panel_open {
        handle.add_css_class("collapsed");
    }
    handle.set_vexpand(true);
    handle.set_valign(Align::Fill);

    let dot = Label::new(Some("·"));
    dot.add_css_class("obsidian-handle-dot");
    dot.set_valign(Align::Center);
    dot.set_vexpand(true);
    handle.append(&dot);

    let gesture = GestureClick::new();
    let revealer_ref = revealer.clone();
    let handle_ref = handle.clone();
    gesture.connect_released(move |_, _, _, _| {
        let opening = !revealer_ref.reveals_child();
        revealer_ref.set_reveal_child(opening);
        if opening {
            handle_ref.remove_css_class("collapsed");
        } else {
            handle_ref.add_css_class("collapsed");
        }
    });
    handle.add_controller(gesture);

    view_row.append(&handle);
    view_row.append(&revealer);
    container.append(&view_row);

    container
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
