mod host;

pub(super) use host::AgentPaneHost;

use std::cell::Cell;
use std::f64::consts::PI;
use std::{cell::RefCell, rc::Rc};

use gtk::{
    glib, prelude::*, Align, Box as GtkBox, Button, DrawingArea, Entry, Label, Orientation,
    PolicyType, ScrolledWindow,
};

use crate::agent::executor::{AgentRuntimeHandle, ExecutorConfig, LogRole};

use super::settings::{self, Settings};

// ─── Network topology ───
// 5-layer deep network: 4 → 7 → 8 → 7 → 3
// Coordinates in normalized [-1, 1] space.

const LAYER_0: &[(f64, f64)] = &[
    (-0.82, -0.52),
    (-0.82, -0.17),
    (-0.82, 0.17),
    (-0.82, 0.52),
];

const LAYER_1: &[(f64, f64)] = &[
    (-0.41, -0.72),
    (-0.41, -0.48),
    (-0.41, -0.24),
    (-0.41, 0.0),
    (-0.41, 0.24),
    (-0.41, 0.48),
    (-0.41, 0.72),
];

const LAYER_2: &[(f64, f64)] = &[
    (0.0, -0.74),
    (0.0, -0.53),
    (0.0, -0.32),
    (0.0, -0.11),
    (0.0, 0.11),
    (0.0, 0.32),
    (0.0, 0.53),
    (0.0, 0.74),
];

const LAYER_3: &[(f64, f64)] = &[
    (0.41, -0.72),
    (0.41, -0.48),
    (0.41, -0.24),
    (0.41, 0.0),
    (0.41, 0.24),
    (0.41, 0.48),
    (0.41, 0.72),
];

const LAYER_4: &[(f64, f64)] = &[
    (0.82, -0.34),
    (0.82, 0.0),
    (0.82, 0.34),
];

const CANVAS_HEIGHT: i32 = 240;
const TICK_MS: u32 = 50;

pub(super) fn build_agent_pane(settings: Rc<RefCell<Settings>>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-agent-pane");
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_overflow(gtk::Overflow::Hidden);

    let runtime = AgentRuntimeHandle::shared(ExecutorConfig {
        confidence_threshold: settings.borrow().agent_confidence_threshold,
        passive_mode: settings.borrow().agent_passive_mode,
    });

    let header = build_header(runtime, settings.clone());
    let brain = build_brain_area();
    let log_area = build_log_area(runtime);
    let pending_bar = build_pending_bar(runtime);
    let prompt = build_prompt_bar(runtime, settings);

    root.append(&header);
    root.append(&brain);
    root.append(&log_area);
    root.append(&pending_bar);
    root.append(&prompt);
    root
}

// ─── Header ───

fn build_header(
    runtime: &'static AgentRuntimeHandle,
    settings: Rc<RefCell<Settings>>,
) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.add_css_class("magma-agent-header");

    let title = Label::new(Some("agent"));
    title.add_css_class("magma-agent-title");
    title.set_halign(Align::Start);

    let status = Label::new(Some("watching"));
    status.add_css_class("magma-agent-status");
    status.set_hexpand(true);
    status.set_halign(Align::End);
    status.set_xalign(1.0);

    let toggle = Button::with_label(if settings.borrow().agent_passive_mode {
        "passive"
    } else {
        "active"
    });
    toggle.add_css_class("magma-agent-toggle");

    {
        let settings_ref = settings;
        toggle.connect_clicked(move |button| {
            let next_passive = !settings_ref.borrow().agent_passive_mode;
            settings_ref.borrow_mut().agent_passive_mode = next_passive;
            settings::save_settings(&settings_ref.borrow());
            runtime.set_passive_mode(next_passive);
            button.set_label(if next_passive { "passive" } else { "active" });
        });
    }

    {
        let status = status.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let snap = runtime.snapshot();
            let text = if snap.status.is_empty() {
                "watching"
            } else {
                &snap.status
            };
            status.set_text(&text.to_lowercase());
            glib::ControlFlow::Continue
        });
    }

    header.append(&title);
    header.append(&status);
    header.append(&toggle);
    header
}

// ─── Canvas ───

fn build_brain_area() -> DrawingArea {
    let canvas = DrawingArea::new();
    canvas.add_css_class("magma-agent-brain");
    canvas.set_hexpand(false);
    canvas.set_vexpand(false);
    canvas.set_halign(Align::Fill);
    canvas.set_valign(Align::Start);
    canvas.set_content_width(380);
    canvas.set_content_height(CANVAS_HEIGHT);

    let phase = Rc::new(Cell::new(0.0_f64));

    {
        let phase = phase.clone();
        canvas.set_draw_func(move |_area, cr, width, height| {
            draw_network(cr, width as f64, height as f64, phase.get());
        });
    }

    {
        let canvas = canvas.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(TICK_MS as u64), move || {
            phase.set(phase.get() + 0.035);
            canvas.queue_draw();
            glib::ControlFlow::Continue
        });
    }

    canvas
}

// ─── Log ───

fn build_log_area(runtime: &'static AgentRuntimeHandle) -> ScrolledWindow {
    let log_box = GtkBox::new(Orientation::Vertical, 0);
    log_box.add_css_class("magma-agent-log");
    log_box.set_vexpand(true);

    let scroll = ScrolledWindow::new();
    scroll.add_css_class("magma-agent-log-scroll");
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_child(Some(&log_box));

    let rendered_count = Rc::new(Cell::new(0_usize));

    {
        let log_box = log_box.clone();
        let scroll = scroll.clone();
        let rendered_count = rendered_count.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let current_len = runtime.log_len();
            if current_len > rendered_count.get() {
                let entries = runtime.log_entries();
                for entry in entries.iter().skip(rendered_count.get()) {
                    let row = build_log_row(entry);
                    log_box.append(&row);
                }
                rendered_count.set(current_len);
                let scroll = scroll.clone();
                glib::idle_add_local_once(move || {
                    let adj = scroll.vadjustment();
                    adj.set_value(adj.upper() - adj.page_size());
                });
            }
            glib::ControlFlow::Continue
        });
    }

    scroll
}

fn build_log_row(entry: &crate::agent::executor::LogEntry) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-agent-log-row");

    let (role_class, role_text, text_class) = match entry.role {
        LogRole::User => ("magma-agent-role-user", "you", "magma-agent-text-user"),
        LogRole::Agent => ("magma-agent-role-agent", "agent", "magma-agent-text-agent"),
        LogRole::System => ("magma-agent-role-system", "sys", "magma-agent-text-system"),
        LogRole::Action => ("magma-agent-role-action", "act", "magma-agent-text-action"),
    };

    let badge = Label::new(Some(role_text));
    badge.add_css_class("magma-agent-log-badge");
    badge.add_css_class(role_class);
    badge.set_valign(Align::Start);

    let text = Label::new(Some(&entry.text));
    text.add_css_class("magma-agent-log-text");
    text.add_css_class(text_class);
    text.set_wrap(true);
    text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text.set_xalign(0.0);
    text.set_hexpand(true);
    text.set_selectable(true);

    row.append(&badge);
    row.append(&text);
    row
}

// ─── Pending bar ───

fn build_pending_bar(runtime: &'static AgentRuntimeHandle) -> GtkBox {
    let bar = GtkBox::new(Orientation::Vertical, 0);
    bar.add_css_class("magma-agent-pending");
    bar.set_visible(false);

    let label = Label::new(Some("proposed action"));
    label.add_css_class("magma-agent-pending-label");
    label.set_halign(Align::Start);

    let detail = Label::new(None);
    detail.add_css_class("magma-agent-pending-text");
    detail.set_wrap(true);
    detail.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    detail.set_xalign(0.0);
    detail.set_hexpand(true);

    let actions = GtkBox::new(Orientation::Horizontal, 6);
    actions.add_css_class("magma-agent-pending-actions");
    actions.set_halign(Align::End);

    let confirm = Button::with_label("confirm");
    confirm.add_css_class("magma-agent-confirm");
    let reject = Button::with_label("reject");
    reject.add_css_class("magma-agent-reject");

    confirm.connect_clicked(move |_| runtime.respond(true));
    reject.connect_clicked(move |_| runtime.respond(false));

    actions.append(&reject);
    actions.append(&confirm);

    bar.append(&label);
    bar.append(&detail);
    bar.append(&actions);

    {
        let bar = bar.clone();
        let detail = detail.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let snap = runtime.snapshot();
            if let Some(dry_run) = &snap.dry_run {
                detail.set_text(dry_run);
                bar.set_visible(true);
            } else {
                bar.set_visible(false);
            }
            glib::ControlFlow::Continue
        });
    }

    bar
}

// ─── Prompt ───

fn build_prompt_bar(
    runtime: &'static AgentRuntimeHandle,
    _settings: Rc<RefCell<Settings>>,
) -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 6);
    bar.add_css_class("magma-agent-prompt");

    let entry = Entry::new();
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("ask agent..."));
    entry.add_css_class("magma-agent-prompt-input");

    let send = Button::with_label("send");
    send.add_css_class("magma-agent-prompt-send");

    {
        let entry = entry.clone();
        send.connect_clicked(move |_| submit_prompt(runtime, &entry));
    }
    entry.connect_activate(move |entry| submit_prompt(runtime, entry));

    bar.append(&entry);
    bar.append(&send);
    bar
}

fn submit_prompt(runtime: &'static AgentRuntimeHandle, entry: &Entry) {
    let text = entry.text().trim().to_string();
    if text.is_empty() {
        return;
    }
    runtime.submit_prompt(text);
    entry.set_text("");
}

// ─── Drawing ───

fn draw_network(cr: &gtk::cairo::Context, w: f64, h: f64, phase: f64) {
    let cx = w * 0.5;
    let cy = h * 0.5;
    let scale = w.min(h) * 0.44;

    let layers: Vec<Vec<(f64, f64)>> = vec![
        layer_points(LAYER_0, cx, cy, scale, phase, 0.0),
        layer_points(LAYER_1, cx, cy, scale, phase, 0.7),
        layer_points(LAYER_2, cx, cy, scale, phase, 1.4),
        layer_points(LAYER_3, cx, cy, scale, phase, 2.1),
        layer_points(LAYER_4, cx, cy, scale, phase, 2.8),
    ];

    // Sparse skip connections first (behind everything).
    draw_skip_edges(cr, &layers[0], &layers[2], phase, 4.0);
    draw_skip_edges(cr, &layers[2], &layers[4], phase, 5.0);

    // Dense layer-to-layer edges.
    for i in 0..4 {
        draw_edges(cr, &layers[i], &layers[i + 1], phase, i as f64 * 0.9);
    }

    // Signals on dense connections.
    for i in 0..4 {
        draw_pulses(cr, &layers[i], &layers[i + 1], phase, i as f64 * 0.9);
    }

    // Nodes on top.
    let offsets = [0.0, 0.7, 1.4, 2.1, 2.8];
    for (i, layer) in layers.iter().enumerate() {
        let is_edge = i == 0 || i == 4;
        draw_nodes(cr, layer, phase, offsets[i], is_edge);
    }
}

fn layer_points(
    pts: &[(f64, f64)],
    cx: f64,
    cy: f64,
    scale: f64,
    phase: f64,
    offset: f64,
) -> Vec<(f64, f64)> {
    pts.iter()
        .enumerate()
        .map(|(i, (x, y))| {
            let t = phase * 0.55 + i as f64 * 0.75 + offset;
            (cx + x * scale + t.sin() * 1.3, cy + y * scale + (t * 0.65).cos() * 1.1)
        })
        .collect()
}

fn draw_edges(
    cr: &gtk::cairo::Context,
    from: &[(f64, f64)],
    to: &[(f64, f64)],
    phase: f64,
    offset: f64,
) {
    cr.set_line_width(0.5);
    for (i, &(x0, y0)) in from.iter().enumerate() {
        for (j, &(x1, y1)) in to.iter().enumerate() {
            let t = (phase * 0.7 + (i * 7 + j) as f64 * 0.5 + offset).sin();
            let alpha = 0.04 + (t * 0.5 + 0.5) * 0.07;
            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            let _ = cr.stroke();
        }
    }
}

fn draw_skip_edges(
    cr: &gtk::cairo::Context,
    from: &[(f64, f64)],
    to: &[(f64, f64)],
    phase: f64,
    offset: f64,
) {
    cr.set_line_width(0.4);
    // Only draw a few sparse skip connections, not fully connected.
    let pairs: &[(usize, usize)] = &[(0, 0), (1, 1), (2, 1), (3, 2)];
    for &(fi, ti) in pairs {
        if fi >= from.len() || ti >= to.len() {
            continue;
        }
        let (x0, y0) = from[fi];
        let (x1, y1) = to[ti];
        let mid_x = (x0 + x1) * 0.5;
        let mid_y = (y0 + y1) * 0.5;
        // Slight curve via control point offset.
        let bend = (phase * 0.3 + offset).sin() * 6.0;
        let t = (phase * 0.6 + fi as f64 * 1.2 + offset).sin();
        let alpha = 0.03 + (t * 0.5 + 0.5) * 0.04;
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
        cr.move_to(x0, y0);
        cr.curve_to(mid_x, mid_y + bend, mid_x, mid_y - bend, x1, y1);
        let _ = cr.stroke();
    }
}

fn draw_pulses(
    cr: &gtk::cairo::Context,
    from: &[(f64, f64)],
    to: &[(f64, f64)],
    phase: f64,
    offset: f64,
) {
    for (i, &(x0, y0)) in from.iter().enumerate() {
        for (j, &(x1, y1)) in to.iter().enumerate() {
            // Stagger: only ~1/3 of connections have a visible pulse at any time.
            let seed = (i * 7 + j * 3) as f64;
            let t = (phase * 0.40 + seed * 0.45 + offset) % (2.0 * PI);
            let frac = t / (2.0 * PI);
            if frac > 0.35 {
                continue;
            }
            let norm = frac / 0.35;
            let px = x0 + (x1 - x0) * norm;
            let py = y0 + (y1 - y0) * norm;
            let alpha = (norm * PI).sin() * 0.45;
            // Subtle dark red tint on some pulses.
            let pulse_tint = ((seed * 0.7 + phase * 0.3).sin() * 0.5 + 0.5) * 0.4;
            cr.set_source_rgba(1.0, 1.0 - pulse_tint * 0.6, 1.0 - pulse_tint * 0.6, alpha);
            cr.arc(px, py, 1.5, 0.0, 2.0 * PI);
            let _ = cr.fill();
        }
    }
}

fn draw_nodes(
    cr: &gtk::cairo::Context,
    nodes: &[(f64, f64)],
    phase: f64,
    offset: f64,
    is_edge_layer: bool,
) {
    for (i, &(x, y)) in nodes.iter().enumerate() {
        let t = (phase * 1.3 + i as f64 * 0.85 + offset).sin();
        let brightness = 0.45 + (t * 0.5 + 0.5) * 0.35;

        // Hidden layers get a dark red tint; edge layers stay white.
        let (r, g, b) = if is_edge_layer {
            (1.0, 1.0, 1.0)
        } else {
            // Dark red accent: #8B2020 at varying blend.
            let red_mix = 0.35 + (t * 0.5 + 0.5) * 0.25;
            (1.0, 1.0 - red_mix * 0.75, 1.0 - red_mix * 0.75)
        };

        // Outer ring — slightly larger for edge layers.
        let ring_r = if is_edge_layer { 5.5 } else { 4.8 };
        cr.set_source_rgba(r, g, b, brightness * 0.20);
        cr.set_line_width(0.8);
        cr.arc(x, y, ring_r, 0.0, 2.0 * PI);
        let _ = cr.stroke();

        // Core.
        let core_r = if is_edge_layer { 2.4 } else { 2.0 };
        cr.set_source_rgba(r, g, b, brightness);
        cr.arc(x, y, core_r, 0.0, 2.0 * PI);
        let _ = cr.fill();
    }
}
