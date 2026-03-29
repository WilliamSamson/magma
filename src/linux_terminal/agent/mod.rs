mod host;

pub(super) use host::AgentPaneHost;

use std::{cell::Cell, rc::Rc};
use std::{cell::RefCell, collections::VecDeque};

use gtk::{
    glib, prelude::*, Align, Box as GtkBox, Button, Entry, Label, Orientation, PolicyType,
    ScrolledWindow,
};
use vte4::prelude::*;

use crate::agent::executor::{AgentRuntimeHandle, ExecutorConfig, LogEntry, LogRole};

use super::{
    persist::SessionSnapshot,
    profile::ProfileId,
    settings::Settings,
    shell,
    terminal,
};

pub(super) fn build_agent_pane(
    settings: Rc<RefCell<Settings>>,
    command_slot: Rc<RefCell<VecDeque<String>>>,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-agent-pane");
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_overflow(gtk::Overflow::Hidden);

    let runtime = AgentRuntimeHandle::shared(ExecutorConfig {
        confidence_threshold: settings.borrow().agent_confidence_threshold,
        passive_mode: settings.borrow().agent_passive_mode,
    });

    root.append(&build_header(runtime));
    root.append(&build_terminal_intro());
    root.append(&build_log_area(runtime));
    root.append(&build_agent_terminal(&settings, command_slot));
    root.append(&build_pending_bar(runtime));
    root.append(&build_command_bar(runtime));
    root
}

fn build_header(runtime: &'static AgentRuntimeHandle) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.add_css_class("magma-agent-header");

    let title = Label::new(Some("agents"));
    title.add_css_class("magma-agent-title");
    title.set_halign(Align::Start);

    let status = Label::new(Some("idle"));
    status.add_css_class("magma-agent-status");
    status.set_hexpand(true);
    status.set_halign(Align::End);
    status.set_xalign(1.0);

    {
        let status = status.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let snapshot = runtime.snapshot();
            let text = if snapshot.status.is_empty() {
                "idle".to_string()
            } else {
                snapshot.status.to_lowercase()
            };
            status.set_text(&text);
            glib::ControlFlow::Continue
        });
    }

    header.append(&title);
    header.append(&status);
    header
}

fn build_terminal_intro() -> GtkBox {
    let panel = GtkBox::new(Orientation::Vertical, 4);
    panel.add_css_class("magma-agent-console");

    let title = Label::new(Some("command terminal"));
    title.add_css_class("magma-agent-console-title");
    title.set_xalign(0.0);

    let body = Label::new(Some(
        "Type an installed agent command and press Enter. Example: codex, claude, gemini, or a full CLI command.",
    ));
    body.add_css_class("magma-agent-console-body");
    body.set_wrap(true);
    body.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    body.set_xalign(0.0);

    panel.append(&title);
    panel.append(&body);
    panel
}

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
                    log_box.append(&build_log_row(entry));
                }
                rendered_count.set(current_len);
                let scroll = scroll.clone();
                glib::idle_add_local_once(move || {
                    let adjustment = scroll.vadjustment();
                    adjustment.set_value(adjustment.upper() - adjustment.page_size());
                });
            }
            glib::ControlFlow::Continue
        });
    }

    scroll
}

fn build_agent_terminal(
    settings: &Rc<RefCell<Settings>>,
    command_slot: Rc<RefCell<VecDeque<String>>>,
) -> GtkBox {
    let shell_box = GtkBox::new(Orientation::Vertical, 0);
    shell_box.add_css_class("magma-agent-shell");
    shell_box.set_vexpand(true);

    let terminal = terminal::build_terminal(ProfileId::Compact, &settings.borrow());
    terminal.add_css_class("magma-agent-shell-terminal");
    let session = SessionSnapshot::new(None);
    let _runtime = shell::spawn_shell(&terminal, &session, &settings.borrow().shell);
    shell_box.append(&terminal);

    {
        let terminal = terminal.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            while let Some(command) = command_slot.borrow_mut().pop_front() {
                terminal.feed_child(command.as_bytes());
                terminal.feed_child(b"\n");
                terminal.grab_focus();
            }
            glib::ControlFlow::Continue
        });
    }

    shell_box
}

fn build_log_row(entry: &LogEntry) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-agent-log-row");

    let (role_class, role_text, text_class) = match entry.role {
        LogRole::User => ("magma-agent-role-user", "$", "magma-agent-text-user"),
        LogRole::Agent => ("magma-agent-role-agent", ">", "magma-agent-text-agent"),
        LogRole::System => ("magma-agent-role-system", "!", "magma-agent-text-system"),
        LogRole::Action => ("magma-agent-role-action", "*", "magma-agent-text-action"),
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

fn build_pending_bar(runtime: &'static AgentRuntimeHandle) -> GtkBox {
    let bar = GtkBox::new(Orientation::Vertical, 0);
    bar.add_css_class("magma-agent-pending");
    bar.set_visible(false);

    let label = Label::new(Some("pending command"));
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

    let confirm = Button::with_label("run");
    confirm.add_css_class("magma-agent-confirm");
    let reject = Button::with_label("cancel");
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
            let snapshot = runtime.snapshot();
            if let Some(dry_run) = &snapshot.dry_run {
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

fn build_command_bar(runtime: &'static AgentRuntimeHandle) -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 8);
    bar.add_css_class("magma-agent-prompt");

    let prompt = Label::new(Some("$"));
    prompt.add_css_class("magma-agent-prompt-glyph");
    prompt.set_valign(Align::Center);

    let entry = Entry::new();
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("codex \"fix the failing tests\""));
    entry.add_css_class("magma-agent-prompt-input");

    let send = Button::with_label("enter");
    send.add_css_class("magma-agent-prompt-send");

    {
        let entry = entry.clone();
        send.connect_clicked(move |_| submit_command(runtime, &entry));
    }
    entry.connect_activate(move |entry| submit_command(runtime, entry));

    bar.append(&prompt);
    bar.append(&entry);
    bar.append(&send);
    bar
}

fn submit_command(runtime: &'static AgentRuntimeHandle, entry: &Entry) {
    let command = entry.text().trim().to_string();
    if command.is_empty() {
        return;
    }
    runtime.submit_command(command);
    entry.set_text("");
}
