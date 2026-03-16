mod app;
mod agent;
mod features;
mod linux_terminal;
mod logger;
mod renderer;
mod ui;
mod window_state;

use std::{io, rc::Rc, time::{Duration, Instant}};

use app::App;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use features::logs::{load_source, parse_args, spawn_file_follower, LogsFeature};
use renderer::Renderer;
use ui::titlebar::{self, TitleBarAction};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{CursorIcon, Icon, ResizeDirection, WindowBuilder},
};

const WINDOW_WIDTH: f64 = 1280.0;
const WINDOW_HEIGHT: f64 = 720.0;
const TICK_RATE: Duration = Duration::from_millis(16); // ~60 FPS for responsive terminal
const RESIZE_BORDER: i32 = 8;
const LOGO_DARK_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon_64.png"));

#[derive(Default)]
struct WindowChrome {
    cursor_position: (f64, f64),
    modifiers: ModifiersState,
    resize_direction: Option<ResizeDirection>,
    titlebar_hover: Option<TitleBarAction>,
    is_focused: bool,
}

fn main() -> io::Result<()> {
    logger::init();

    let args = parse_args()?;

    if args.input_path.is_none() {
        logger::info("launching terminal mode", &[]);
        return linux_terminal::run();
    }

    let source = load_source(args.input_path)?;
    logger::info("log source loaded", &[
        ("source", &source.source_name),
        ("entries", &source.entries.len().to_string()),
    ]);
    let follower = source.follow_config.map(spawn_file_follower);
    let mut logs = LogsFeature::new(source.source_name, source.entries, follower);
    logs.apply_startup_filters(args.startup_filter.query, &args.startup_filter.levels);
    let mut app = App::new_logs(logs);

    let event_loop = EventLoop::new().map_err(to_io_error)?;
    let (icon, icon_rgba, icon_w, icon_h) = build_icon()?;
    let initial_size = match window_state::load_window_size() {
        Ok(Some(size)) => size,
        Ok(None) => PhysicalSize::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        Err(error) => {
            eprintln!("window size load failed: {error}");
            PhysicalSize::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        }
    };
    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Magma")
            .with_inner_size(Size::Physical(initial_size))
            .with_window_icon(Some(icon))
            .with_decorations(false) // Custom title bar — we draw our own.
            .build(&event_loop)
            .map_err(to_io_error)?,
    );
    let _ = window.request_inner_size(Size::Physical(initial_size));
    let mut renderer = Renderer::new(window.clone(), icon_rgba, icon_w, icon_h)
        .map_err(to_io_error)?;
    let mut next_tick = Instant::now();
    let mut chrome = WindowChrome::default();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::WaitUntil(next_tick));

            match event {
                Event::AboutToWait => handle_tick(&window, &mut app, &mut next_tick, elwt),
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    handle_window_event(
                        event,
                        &window,
                        &mut renderer,
                        &mut app,
                        elwt,
                        &mut chrome,
                    );
                }
                _ => {}
            }
        })
        .map_err(to_io_error)
}

fn handle_tick(
    window: &winit::window::Window,
    app: &mut App,
    next_tick: &mut Instant,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
) {
    let now = Instant::now();
    if now < *next_tick {
        return;
    }

    app.tick();
    if app.should_quit() {
        persist_window_size(window);
        elwt.exit();
        return;
    }

    window.request_redraw();
    *next_tick = now + TICK_RATE;
}

fn handle_window_event(
    event: WindowEvent,
    window: &winit::window::Window,
    renderer: &mut Renderer,
    app: &mut App,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    chrome: &mut WindowChrome,
) {
    match event {
        WindowEvent::CloseRequested => {
            persist_window_size(window);
            elwt.exit();
        }
        WindowEvent::Focused(focused) => {
            chrome.is_focused = focused;
            window.request_redraw();
        }
        WindowEvent::Resized(size) => {
            if let Err(error) = renderer.resize(size.width, size.height) {
                eprintln!("resize failed: {error}");
                elwt.exit();
                return;
            }
            if !window.is_maximized() {
                if let Err(error) = window_state::save_window_size(size) {
                    eprintln!("window size save failed: {error}");
                }
            }
            window.request_redraw();
        }
        WindowEvent::CursorMoved { position, .. } => {
            chrome.cursor_position = (position.x, position.y);
            chrome.resize_direction = resize_direction_at_point(
                chrome.cursor_position.0,
                chrome.cursor_position.1,
                window.inner_size(),
                window.is_maximized(),
            );
            window.set_cursor_icon(cursor_icon_for_resize(chrome.resize_direction));
            renderer.update_dock_hover(position.x as i32, position.y as i32);
            
            let new_hover = titlebar::hit_test(position.x, position.y, window.inner_size().width);
            let hover_action = if new_hover != TitleBarAction::Drag && new_hover != TitleBarAction::None {
                Some(new_hover)
            } else {
                None
            };
            
            if chrome.titlebar_hover != hover_action {
                chrome.titlebar_hover = hover_action;
                window.request_redraw();
            } else {
                // Dock might need redraw if it handles its own hover
                window.request_redraw(); 
            }
        }
        WindowEvent::CursorLeft { .. } => {
            chrome.resize_direction = None;
            if chrome.titlebar_hover.is_some() {
                chrome.titlebar_hover = None;
                window.request_redraw();
            }
            window.set_cursor_icon(CursorIcon::Default);
        }
        WindowEvent::ModifiersChanged(modifiers) => {
            chrome.modifiers = modifiers.state();
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            if let Some(direction) = chrome.resize_direction {
                let _ = window.drag_resize_window(direction);
                return;
            }

            let (x, y) = chrome.cursor_position;
            
            // Handle bottom pill clicks first
            if let Some(tab_idx) = renderer.dock_hit_test(x as i32, y as i32) {
                renderer.set_active_dock_item(tab_idx);
                window.request_redraw();
                return;
            }

            // Fallback to titlebar if not hit
            match titlebar::hit_test(x, y, window.inner_size().width) {
                TitleBarAction::Close => {
                    persist_window_size(window);
                    elwt.exit();
                }
                TitleBarAction::Minimize => window.set_minimized(true),
                TitleBarAction::Maximize => {
                    window.set_maximized(!window.is_maximized());
                }
                TitleBarAction::Drag => {
                    let _ = window.drag_window();
                }
                TitleBarAction::None => {}
            }
        }
        WindowEvent::KeyboardInput { event, .. } => {
            if let Some(input) = translate_key_event(&event, chrome.modifiers) {
                app.handle_event(input);
                if app.should_quit() {
                    elwt.exit();
                    return;
                }
                window.request_redraw();
            }
        }
        WindowEvent::RedrawRequested => {
            if let Err(error) = redraw(renderer, app, chrome) {
                eprintln!("render failed: {error}");
                elwt.exit();
            }
        }
        _ => {}
    }
}

fn redraw(renderer: &mut Renderer, app: &mut App, chrome: &WindowChrome) -> io::Result<()> {
    let (columns, rows) = renderer.grid_size();
    let buffer = app.render(columns, rows);
    let is_focused = chrome.is_focused;
    renderer.render(&buffer, app.show_dock(), chrome.titlebar_hover, is_focused).map_err(to_io_error)
}

fn translate_key_event(
    event: &winit::event::KeyEvent,
    modifiers: ModifiersState,
) -> Option<CrosstermEvent> {
    if event.state != ElementState::Pressed {
        return None;
    }

    let modifiers = translate_modifiers(modifiers);
    let code = match &event.logical_key {
        Key::Named(NamedKey::ArrowUp) => KeyCode::Up,
        Key::Named(NamedKey::ArrowDown) => KeyCode::Down,
        Key::Named(NamedKey::ArrowLeft) => KeyCode::Left,
        Key::Named(NamedKey::ArrowRight) => KeyCode::Right,
        Key::Named(NamedKey::PageUp) => KeyCode::PageUp,
        Key::Named(NamedKey::PageDown) => KeyCode::PageDown,
        Key::Named(NamedKey::Home) => KeyCode::Home,
        Key::Named(NamedKey::End) => KeyCode::End,
        Key::Named(NamedKey::Enter) => KeyCode::Enter,
        Key::Named(NamedKey::Escape) => KeyCode::Esc,
        Key::Named(NamedKey::Backspace) => KeyCode::Backspace,
        Key::Named(NamedKey::Tab) => KeyCode::Tab,
        Key::Named(NamedKey::Delete) => KeyCode::Delete,
        Key::Named(NamedKey::Space) => KeyCode::Char(' '),
        Key::Character(text) => KeyCode::Char(text.chars().next()?),
        _ => KeyCode::Char(first_printable_char(event.text.as_deref())?),
    };

    Some(CrosstermEvent::Key(KeyEvent::new_with_kind(
        code,
        modifiers,
        KeyEventKind::Press,
    )))
}

/// Returns (Icon, raw_rgba, width, height) — the RGBA data is reused for the title bar icon.
fn build_icon() -> io::Result<(Icon, Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(header_logo_bytes())
        .map_err(to_io_error)?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    let icon = Icon::from_rgba(rgba.clone(), width, height).map_err(to_io_error)?;
    Ok((icon, rgba, width, height))
}

fn header_logo_bytes() -> &'static [u8] {
    LOGO_DARK_BYTES
}

fn translate_modifiers(modifiers: ModifiersState) -> KeyModifiers {
    let mut result = KeyModifiers::NONE;
    if modifiers.shift_key() {
        result |= KeyModifiers::SHIFT;
    }
    if modifiers.control_key() {
        result |= KeyModifiers::CONTROL;
    }
    if modifiers.alt_key() {
        result |= KeyModifiers::ALT;
    }
    result
}

fn first_printable_char(text: Option<&str>) -> Option<char> {
    text?.chars().find(|character| *character == ' ' || !character.is_control())
}

fn to_io_error(error: impl ToString) -> io::Error {
    io::Error::other(error.to_string())
}

fn persist_window_size(window: &winit::window::Window) {
    if window.is_maximized() {
        return;
    }

    if let Err(error) = window_state::save_window_size(window.inner_size()) {
        eprintln!("window size save failed: {error}");
    }
}

fn resize_direction_at_point(
    x: f64,
    y: f64,
    size: PhysicalSize<u32>,
    maximized: bool,
) -> Option<ResizeDirection> {
    if maximized || size.width == 0 || size.height == 0 {
        return None;
    }

    let x = x as i32;
    let y = y as i32;
    let width = size.width as i32;
    let height = size.height as i32;

    let left = (0..RESIZE_BORDER).contains(&x);
    let right = x >= width - RESIZE_BORDER && x < width;
    let top = (0..RESIZE_BORDER).contains(&y);
    let bottom = y >= height - RESIZE_BORDER && y < height;

    match (left, right, top, bottom) {
        (true, false, true, false) => Some(ResizeDirection::NorthWest),
        (false, true, true, false) => Some(ResizeDirection::NorthEast),
        (true, false, false, true) => Some(ResizeDirection::SouthWest),
        (false, true, false, true) => Some(ResizeDirection::SouthEast),
        (true, false, false, false) => Some(ResizeDirection::West),
        (false, true, false, false) => Some(ResizeDirection::East),
        (false, false, true, false) => Some(ResizeDirection::North),
        (false, false, false, true) => Some(ResizeDirection::South),
        _ => None,
    }
}

fn cursor_icon_for_resize(direction: Option<ResizeDirection>) -> CursorIcon {
    match direction {
        Some(ResizeDirection::North) | Some(ResizeDirection::South) => CursorIcon::NsResize,
        Some(ResizeDirection::East) | Some(ResizeDirection::West) => CursorIcon::EwResize,
        Some(ResizeDirection::NorthWest) | Some(ResizeDirection::SouthEast) => CursorIcon::NwseResize,
        Some(ResizeDirection::NorthEast) | Some(ResizeDirection::SouthWest) => CursorIcon::NeswResize,
        None => CursorIcon::Default,
    }
}
