mod agent;
mod folder;
pub(crate) mod git;
mod header;
mod input;
mod left_pane;
mod logr;
mod meta;
mod notes;
pub(crate) mod persist;
mod right_pane;
pub(crate) mod settings;
mod setup;
mod style;
mod terminal;
pub(crate) mod theme;
mod view;
mod web;
mod workspace;

use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

use gtk::{
    Application, ApplicationWindow, Box as GtkBox, IconTheme, Orientation, Stack,
    StackTransitionType, gdk, gio, glib, prelude::*,
};
use std::{cell::RefCell, rc::Rc};
use winit::dpi::PhysicalSize;

use crate::window_state;

const APP_ID: &str = "io.magma.terminal";
const APP_TITLE: &str = "Magma";
const HEADER_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon_64.png");
const ICON_THEME_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
const MARGIN_HORIZONTAL: i32 = 16;
const MARGIN_TOP: i32 = 4;
const MARGIN_BOTTOM: i32 = 16;

pub(crate) fn run() -> io::Result<()> {
    let initial_size = window_state::load_window_size()?.unwrap_or_default();
    configure_webkit_runtime();
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

fn configure_webkit_runtime() {
    if !should_disable_webkit_sandbox() {
        return;
    }

    if let Err(error) = glib::setenv("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1", true) {
        eprintln!("webkit sandbox override failed: {error}");
    }
}

fn should_disable_webkit_sandbox() -> bool {
    if std::env::var_os("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS").is_some() {
        return true;
    }

    if std::env::var_os("APPDIR").is_some() {
        return true;
    }

    // WebKit aborts the application if its bubblewrap sandbox cannot create
    // user namespaces, so disable it only when this host cannot support it.
    !webkit_sandbox_supported()
}

fn webkit_sandbox_supported() -> bool {
    if !command_available("xdg-dbus-proxy") {
        return false;
    }

    let true_path = if Path::new("/usr/bin/true").exists() {
        "/usr/bin/true"
    } else {
        "/bin/true"
    };

    let mut probe = Command::new("bwrap");
    probe
        .arg("--unshare-user")
        .arg("--uid")
        .arg("0")
        .arg("--gid")
        .arg("0")
        .arg("--proc")
        .arg("/proc")
        .arg("--dev")
        .arg("/dev");

    for path in ["/usr", "/bin", "/lib", "/lib64"] {
        if Path::new(path).exists() {
            probe.arg("--ro-bind").arg(path).arg(path);
        }
    }

    probe
        .arg(true_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn command_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn build_window(app: &Application, width: u32, height: u32) {
    // Rc<RefCell<Settings>> shares the mutable runtime settings across settings UI and workspace callbacks on the GTK thread.
    let app_settings = Rc::new(RefCell::new(settings::load_settings()));
    let needs_setup = !settings::settings_exist();
    let initial_setup_step = if needs_setup {
        let (checkpoint_settings, checkpoint_step) = setup::load_checkpoint(&app_settings.borrow());
        *app_settings.borrow_mut() = checkpoint_settings;
        checkpoint_step
    } else {
        setup::clear_checkpoint();
        0
    };

    // Register bundled icon theme so the taskbar/desktop can find the app icon
    if let Some(display) = gdk::Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        icon_theme.add_search_path(ICON_THEME_PATH);
    }

    style::install_css(&app_settings.borrow());

    let header = header::build_header();
    let workspace = std::rc::Rc::new(workspace::WorkspaceView::new(app_settings.clone()));
    {
        let workspace_ref = workspace.clone();
        glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            workspace_ref.save();
            glib::ControlFlow::Continue
        });
    }
    // Rc<dyn Fn()> is the lightest way to let side panes query the active terminal cwd on the GTK thread without owning workspace internals.
    let cwd_provider: Rc<dyn Fn() -> Option<String>> = {
        let workspace_ref = workspace.clone();
        Rc::new(move || workspace_ref.current_cwd())
    };
    let terminal_provider: Rc<dyn Fn() -> Option<vte4::Terminal>> = {
        let workspace_ref = workspace.clone();
        Rc::new(move || workspace_ref.current_terminal())
    };
    header::wire_inspector(
        header.inspector_button(),
        terminal_provider.clone(),
        app_settings.clone(),
    );
    let shell = shell_container(workspace.root(), app_settings.clone(), cwd_provider);

    // Stack: workspace (main) <-> settings
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    stack.add_named(shell.root(), Some("workspace"));

    {
        let stack_ref = stack.clone();
        let settings_ref = app_settings.clone();
        let workspace_ref = workspace.clone();
        let shell_ref = shell.clone();
        let setup_page = setup::build_setup_page(
            &app_settings.borrow(),
            initial_setup_step,
            style::install_css,
            move |configured_settings| {
                *settings_ref.borrow_mut() = configured_settings.clone();
                style::install_css(&configured_settings);
                workspace_ref.apply_settings(&configured_settings);
                shell_ref.apply_settings(&configured_settings);
                // Synchronize the renderer theme palette when setup configuration changes
                crate::ui::theme::set_palette(theme::palette(configured_settings.theme_mode));
                let snapshot = settings_ref.borrow().clone();
                settings::save_settings(&snapshot);
                setup::clear_checkpoint();
                stack_ref.set_visible_child_name("workspace");
            },
        );
        stack.add_named(&setup_page, Some("setup"));
    }

    let settings_host = GtkBox::new(Orientation::Vertical, 0);
    settings_host.set_hexpand(true);
    settings_host.set_vexpand(true);
    stack.add_named(&settings_host, Some("settings"));
    stack.set_visible_child_name(if needs_setup { "setup" } else { "workspace" });

    {
        let header_ref = header.clone();
        stack.connect_visible_child_name_notify(move |stack| {
            match stack.visible_child_name().as_deref() {
                Some("settings") => header_ref.show_settings_mode(),
                Some("setup") => header_ref.show_workspace_mode(false, false),
                _ => header_ref.show_workspace_mode(true, true),
            }
        });
    }
    if needs_setup {
        header.show_workspace_mode(false, false);
    } else {
        header.show_workspace_mode(true, true);
    }

    // Settings button toggles to settings view
    {
        let stack_ref = stack.clone();
        let settings_host = settings_host.clone();
        let settings_ref = app_settings.clone();
        let workspace_ref = workspace.clone();
        let shell_ref = shell.clone();
        let header_ref = header.clone();
        header.settings_button().connect_clicked(move |_| {
            let current = stack_ref.visible_child_name();
            if current.as_deref() == Some("settings") {
                stack_ref.set_visible_child_name("workspace");
            } else {
                mount_settings_page(
                    &settings_host,
                    settings_ref.clone(),
                    {
                        let stack_ref = stack_ref.clone();
                        move || {
                            stack_ref.set_visible_child_name("workspace");
                        }
                    },
                    {
                        let header_ref = header_ref.clone();
                        move |title| {
                            header_ref.set_settings_title(title);
                        }
                    },
                    {
                        let header_ref = header_ref.clone();
                        move |action| {
                            header_ref.set_settings_close_action(action);
                        }
                    },
                    {
                        let settings_ref = settings_ref.clone();
                        let workspace_ref = workspace_ref.clone();
                        let shell_ref = shell_ref.clone();
                        move |new_settings| {
                            *settings_ref.borrow_mut() = new_settings.clone();
                            style::install_css(new_settings);
                            workspace_ref.apply_settings(new_settings);
                            shell_ref.apply_settings(new_settings);
                            // Synchronize the renderer theme palette when workspace settings are applied
                            crate::ui::theme::set_palette(theme::palette(new_settings.theme_mode));
                        }
                    },
                    {
                        let shell_ref = shell_ref.clone();
                        move || {
                            shell_ref.clear_web_data();
                        }
                    },
                );
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
    window.add_css_class("magma-window");
    window.set_titlebar(Some(header.widget()));
    window.set_child(Some(&stack));

    let close_stack = stack.clone();
    window.connect_close_request(move |window| {
        if close_stack.visible_child_name().as_deref() == Some("settings") {
            close_stack.set_visible_child_name("workspace");
            return glib::Propagation::Stop;
        }

        workspace.save();
        persist_window_size(window);
        glib::Propagation::Proceed
    });

    window.present();
}

#[derive(Clone)]
#[allow(dead_code)]
struct ShellContainer {
    root: GtkBox,
    left_pane: left_pane::LeftPane,
    side_panes: right_pane::SidePanes,
}

impl ShellContainer {
    fn root(&self) -> &GtkBox {
        &self.root
    }

    fn apply_settings(&self, settings: &settings::Settings) {
        self.side_panes.apply_settings(settings);
    }

    fn clear_web_data(&self) {
        self.side_panes.clear_web_data();
    }
}

fn shell_container(
    child: &impl IsA<gtk::Widget>,
    settings: Rc<RefCell<settings::Settings>>,
    cwd_provider: Rc<dyn Fn() -> Option<String>>,
) -> ShellContainer {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("magma-shell");
    container.set_margin_start(MARGIN_HORIZONTAL);
    container.set_margin_end(MARGIN_HORIZONTAL);
    container.set_margin_top(MARGIN_TOP);
    container.set_margin_bottom(MARGIN_BOTTOM);

    let view_row = GtkBox::new(Orientation::Horizontal, 0);
    view_row.set_spacing(2);
    view_row.set_vexpand(true);

    // Deferred side_panes reference so the file-click callback can reach it
    let side_panes_slot: Rc<RefCell<Option<right_pane::SidePanes>>> = Rc::new(RefCell::new(None));

    // Left pane: folder revealer + handle (handle sits to the right of the pane)
    let on_file_click: Rc<dyn Fn(&std::path::Path)> = {
        let slot = side_panes_slot.clone();
        Rc::new(move |path: &std::path::Path| {
            if let Some(sp) = slot.borrow().as_ref() {
                sp.open_view_file(path);
            }
        })
    };
    let left_pane = left_pane::build_left_pane(cwd_provider.clone(), on_file_click);
    view_row.append(left_pane.revealer());
    view_row.append(left_pane.handle());

    // Workspace (terminal) sits in the center
    view_row.append(child);

    // Right pane: handle + feature revealers
    let side_panes = right_pane::build_side_panes(settings.clone(), cwd_provider);
    *side_panes_slot.borrow_mut() = Some(side_panes.clone());
    view_row.append(side_panes.handle());
    view_row.append(side_panes.logr_revealer());
    view_row.append(side_panes.web_revealer());
    view_row.append(side_panes.view_revealer());
    view_row.append(side_panes.git_revealer());
    view_row.append(side_panes.agent_revealer());
    view_row.append(side_panes.notes_revealer());
    container.append(&view_row);

    ShellContainer {
        root: container,
        left_pane,
        side_panes,
    }
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

fn mount_settings_page(
    host: &GtkBox,
    settings: Rc<RefCell<settings::Settings>>,
    on_back: impl Fn() + 'static,
    on_title_change: impl Fn(&str) + 'static,
    on_close_action_change: impl Fn(Rc<dyn Fn()>) + 'static,
    on_apply: impl Fn(&settings::Settings) + 'static,
    on_clear_browser_data: impl Fn() + 'static,
) {
    while let Some(child) = host.first_child() {
        host.remove(&child);
    }

    let page = settings::build_settings_page(
        settings,
        on_back,
        on_title_change,
        on_close_action_change,
        on_apply,
        on_clear_browser_data,
    );
    host.append(&page);
}
