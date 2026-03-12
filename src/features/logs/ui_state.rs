use std::sync::mpsc::Receiver;

use super::log_entry::LogEntry;

pub(crate) struct UiState {
    pub(crate) help_visible: bool,
    pub(crate) last_action: Option<String>,
}

pub(crate) struct ViewState {
    pub(crate) scroll: usize,
    pub(crate) viewport_height: usize,
}

pub(crate) struct SourceState {
    pub(crate) name: String,
    pub(crate) follower: Option<Receiver<LogEntry>>,
}
