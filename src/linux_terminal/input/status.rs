use std::{
    cell::{Cell, RefCell},
    fs,
    path::Path,
    rc::Rc,
};

use gtk::{gio, glib, prelude::*, Label};

pub(super) struct StatusWidgets {
    pub(super) status: Label,
    pub(super) notice: Label,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StatusEvent {
    sequence: u64,
    status_code: i32,
    command: String,
}

pub(super) fn build_status_widgets(status_path: &Path, notifications: Rc<Cell<bool>>) -> StatusWidgets {
    let status = Label::new(None);
    status.add_css_class("magma-status-label");
    status.add_css_class("magma-status-ok");

    let notice = Label::new(None);
    notice.add_css_class("magma-notice-label");
    notice.set_visible(false);

    let initial = read_status_event(status_path);
    update_status_label(&status, initial.status_code);
    watch_status_file(&status, &notice, status_path, initial, notifications);

    StatusWidgets { status, notice }
}

fn watch_status_file(
    status: &Label,
    notice: &Label,
    status_path: &Path,
    initial: StatusEvent,
    notifications: Rc<Cell<bool>>,
) {
    let status = status.clone();
    let notice = notice.clone();
    let status_path = status_path.to_path_buf();
    let last_event = Rc::new(RefCell::new(initial));
    let notice_version = Rc::new(Cell::new(0u64));

    glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
        let next_event = read_status_event(&status_path);
        let mut last_event_ref = last_event.borrow_mut();
        if next_event == *last_event_ref {
            return glib::ControlFlow::Continue;
        }

        update_status_label(&status, next_event.status_code);
        if next_event.sequence != last_event_ref.sequence && !next_event.command.trim().is_empty() {
            show_notice(&notice, &notice_version, &next_event);
            if notifications.get() {
                show_desktop_notice(&next_event);
            }
        }

        *last_event_ref = next_event;
        glib::ControlFlow::Continue
    });
}

fn read_status_event(status_path: &Path) -> StatusEvent {
    let raw = fs::read_to_string(status_path).unwrap_or_default();
    let line = raw.trim();

    if let Some((sequence, status_code, command)) = parse_status_triplet(line) {
        return StatusEvent {
            sequence,
            status_code,
            command,
        };
    }

    StatusEvent {
        sequence: 0,
        status_code: line.parse::<i32>().unwrap_or(0),
        command: String::new(),
    }
}

fn parse_status_triplet(line: &str) -> Option<(u64, i32, String)> {
    let mut parts = line.splitn(3, '\t');
    let sequence = parts.next()?.parse::<u64>().ok()?;
    let status_code = parts.next()?.parse::<i32>().ok()?;
    let command = parts.next().unwrap_or_default().to_string();
    Some((sequence, status_code, command))
}

fn update_status_label(label: &Label, status_code: i32) {
    let text = if status_code == 0 {
        "ready".to_string()
    } else {
        format!("failed {status_code}")
    };
    label.set_text(&text);
    label.remove_css_class("magma-status-ok");
    label.remove_css_class("magma-status-error");
    if status_code == 0 {
        label.add_css_class("magma-status-ok");
    } else {
        label.add_css_class("magma-status-error");
    }
}

fn show_notice(notice: &Label, notice_version: &Cell<u64>, event: &StatusEvent) {
    let version = notice_version.get().saturating_add(1);
    notice_version.set(version);

    let (message, class_name) = if event.status_code == 0 {
        (format!("done · {}", compact_command(&event.command)), "magma-notice-ok")
    } else {
        (
            format!(
                "failed {} · {}",
                event.status_code,
                compact_command(&event.command)
            ),
            "magma-notice-error",
        )
    };

    notice.set_text(&message);
    notice.remove_css_class("magma-notice-ok");
    notice.remove_css_class("magma-notice-error");
    notice.add_css_class(class_name);
    notice.set_visible(true);

    let notice = notice.clone();
    let notice_version = notice_version.clone();
    glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
        if notice_version.get() != version {
            return;
        }
        notice.set_visible(false);
    });
}

fn show_desktop_notice(event: &StatusEvent) {
    let Some(application) = gio::Application::default() else {
        return;
    };

    let (title, body) = if event.status_code == 0 {
        (
            "Command completed".to_string(),
            format!("done · {}", compact_command(&event.command)),
        )
    } else {
        (
            format!("Command failed ({})", event.status_code),
            compact_command(&event.command),
        )
    };

    let notification = gio::Notification::new(&title);
    notification.set_body(Some(&body));
    notification.set_icon(&gio::ThemedIcon::new("utilities-terminal"));
    if event.status_code != 0 {
        notification.set_priority(gio::NotificationPriority::High);
    }

    application.send_notification(Some(&format!("command-{}", event.sequence)), &notification);
}

fn compact_command(command: &str) -> String {
    let compact = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 28 {
        return compact;
    }

    let shortened = compact.chars().take(25).collect::<String>();
    format!("{shortened}...")
}
