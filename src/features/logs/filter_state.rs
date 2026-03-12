use crossterm::event::KeyCode;

use super::{level_filters::LevelFilters, log_entry::LogEntry, log_level::LogLevel};

pub(crate) struct FilterState {
    query: String,
    mode: InputMode,
    levels: LevelFilters,
}

enum InputMode {
    Normal,
    Search,
}

impl FilterState {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            mode: InputMode::Normal,
            levels: LevelFilters::new(),
        }
    }

    pub(crate) fn matches(&self, entry: &LogEntry) -> bool {
        self.levels.matches(entry.level()) && entry.matches_query(&self.query)
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match self.mode {
            InputMode::Search => self.handle_search_key(code),
            InputMode::Normal => self.handle_normal_key(code),
        }
    }

    pub(crate) fn status_line(&self) -> String {
        let mode = match self.mode {
            InputMode::Normal => "nav",
            InputMode::Search => "search",
        };

        format!(
            "mode: {mode}  query: {}  levels: {}",
            self.query_text(),
            self.levels.summary()
        )
    }

    pub(crate) fn set_query(&mut self, query: String) {
        self.query = query;
        self.mode = InputMode::Normal;
    }

    pub(crate) fn set_levels(&mut self, levels: &[LogLevel]) {
        self.levels.set_active_levels(levels);
    }

    fn handle_normal_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('/') => {
                self.mode = InputMode::Search;
                true
            }
            KeyCode::Char('c') => {
                self.clear();
                true
            }
            KeyCode::Char('t') => self.toggle(LogLevel::Trace),
            KeyCode::Char('d') => self.toggle(LogLevel::Debug),
            KeyCode::Char('i') => self.toggle(LogLevel::Info),
            KeyCode::Char('w') => self.toggle(LogLevel::Warn),
            KeyCode::Char('e') => self.toggle(LogLevel::Error),
            _ => false,
        }
    }

    fn handle_search_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Enter | KeyCode::Esc => {
                self.mode = InputMode::Normal;
                true
            }
            KeyCode::Backspace => {
                self.query.pop();
                true
            }
            KeyCode::Char(character) => {
                self.query.push(character);
                true
            }
            _ => false,
        }
    }

    fn toggle(&mut self, level: LogLevel) -> bool {
        self.levels.toggle(level);
        true
    }

    fn clear(&mut self) {
        self.query.clear();
        self.levels.clear();
        self.mode = InputMode::Normal;
    }

    fn query_text(&self) -> &str {
        if self.query.is_empty() {
            "<none>"
        } else {
            &self.query
        }
    }
}
