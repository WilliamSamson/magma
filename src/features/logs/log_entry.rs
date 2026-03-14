use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use serde_json::{Map, Value};

use super::log_level::LogLevel;

#[derive(Clone)]
pub(crate) struct LogEntry {
    line_number: usize,
    timestamp: Option<String>,
    level: LogLevel,
    message: String,
    fields: String,
    raw: String,
}

impl LogEntry {
    pub(crate) fn from_json_line(line_number: usize, line: &str) -> Self {
        match serde_json::from_str(line) {
            Ok(value) => Self::from_value(line_number, value),
            Err(error) => Self::malformed(line_number, line, error),
        }
    }

    pub(crate) fn from_value(line_number: usize, value: Value) -> Self {
        match value {
            Value::Object(map) => Self::from_object(line_number, map),
            other => {
                let raw = stringify_value(&other);
                Self {
                    line_number,
                    timestamp: None,
                    level: LogLevel::Unknown,
                    message: raw.clone(),
                    fields: String::new(),
                    raw,
                }
            }
        }
    }

    pub(crate) fn line_number(&self) -> usize {
        self.line_number
    }

    pub(crate) fn level(&self) -> LogLevel {
        self.level
    }

    pub(crate) fn level_label(&self) -> &'static str {
        self.level.label()
    }

    pub(crate) fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn fields_summary(&self) -> &str {
        &self.fields
    }

    pub(crate) fn raw_line(&self) -> &str {
        &self.raw
    }

    pub(crate) fn matches_query(&self, query: &str) -> bool {
        query.is_empty()
            || contains_ignore_case(&self.raw, query)
            || contains_ignore_case(self.level.label(), query)
    }

    pub(crate) fn render_line(&self) -> Line<'static> {
        let mut spans = vec![
            Span::raw(format!("{:>4} ", self.line_number)),
            Span::styled(
                format!("{:<5}", self.level.label()),
                self.level.style().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ];

        if let Some(timestamp) = &self.timestamp {
            spans.push(Span::styled(
                format!("[{timestamp}] "),
                Style::default().fg(Color::Cyan),
            ));
        }

        // `Line<'static>` requires owned text, so the message is cloned into the span here.
        spans.push(Span::raw(self.message.clone()));
        if !self.fields.is_empty() {
            spans.push(Span::styled(
                format!(" {}", self.fields),
                Style::default().fg(Color::DarkGray),
            ));
        }

        Line::from(spans)
    }

    pub(crate) fn follow_error(line_number: usize, message: String) -> Self {
        Self {
            line_number,
            timestamp: None,
            level: LogLevel::Error,
            raw: message.clone(),
            message,
            fields: "source=follower".to_string(),
        }
    }

    fn malformed(line_number: usize, line: &str, error: serde_json::Error) -> Self {
        Self {
            line_number,
            timestamp: None,
            level: LogLevel::Error,
            message: "malformed JSON".to_string(),
            fields: format!("parse_error={error}"),
            raw: line.to_string(),
        }
    }

    fn from_object(line_number: usize, map: Map<String, Value>) -> Self {
        let raw = stringify_object(&map);
        let timestamp = extract_text(&map, &["timestamp", "@timestamp", "time", "ts"]);
        let level = parse_level(&map);
        let message = extract_text(&map, &["message", "msg", "event", "body"])
            .unwrap_or_else(|| raw.clone());
        let fields = summarize_fields(&map);

        Self {
            line_number,
            timestamp,
            level,
            message,
            fields,
            raw,
        }
    }
}

fn parse_level(map: &Map<String, Value>) -> LogLevel {
    extract_text(map, &["level", "severity", "log_level", "lvl"])
        .map(|value| LogLevel::from_text(&value))
        .unwrap_or(LogLevel::Unknown)
}

fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
}

fn extract_text(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| map.get(*key))
        .map(stringify_value)
        .filter(|value| !value.is_empty())
}

fn summarize_fields(map: &Map<String, Value>) -> String {
    let mut pairs = Vec::new();
    for (key, value) in map {
        if is_reserved_key(key.as_str()) {
            continue;
        }

        pairs.push(format!("{key}={}", stringify_value(value)));
        if pairs.len() == 3 {
            break;
        }
    }

    pairs.join(" ")
}

fn is_reserved_key(key: &str) -> bool {
    matches!(
        key,
        "timestamp"
            | "@timestamp"
            | "time"
            | "ts"
            | "level"
            | "severity"
            | "log_level"
            | "lvl"
            | "message"
            | "msg"
            | "event"
            | "body"
    )
}

fn stringify_object(map: &Map<String, Value>) -> String {
    serde_json::to_string(map).unwrap_or_else(|_| "<unprintable json object>".to_string())
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.to_owned(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<unprintable json>".to_string()),
    }
}
