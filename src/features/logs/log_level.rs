use ratatui::style::{Color, Style};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Unknown,
}

impl LogLevel {
    pub(crate) fn from_text(raw_level: &str) -> Self {
        match raw_level.to_ascii_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warn" | "warning" => Self::Warn,
            "error" | "fatal" => Self::Error,
            _ => Self::Unknown,
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Unknown => "UNKWN",
        }
    }

    pub(crate) fn style(&self) -> Style {
        match self {
            Self::Trace => Style::default().fg(Color::DarkGray),
            Self::Debug => Style::default().fg(Color::Blue),
            Self::Info => Style::default().fg(Color::Green),
            Self::Warn => Style::default().fg(Color::Yellow),
            Self::Error => Style::default().fg(Color::Red),
            Self::Unknown => Style::default().fg(Color::Gray),
        }
    }
}
