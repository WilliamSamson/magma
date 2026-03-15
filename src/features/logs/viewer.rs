use std::sync::mpsc::{Receiver, TryRecvError};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{app::ShellContext, ui::{layout, theme}};

use super::{
    export,
    filter_state::FilterState,
    log_entry::LogEntry,
    log_level::LogLevel,
    ui_state::{SourceState, UiState, ViewState},
};

pub(crate) struct LogsFeature {
    entries: Vec<LogEntry>,
    view: ViewState,
    should_quit: bool,
    source: SourceState,
    filters: FilterState,
    ui: UiState,
}

impl LogsFeature {
    const EXPORT_PATH: &str = "magma-export.jsonl";
    pub(crate) fn new(
        source_name: String,
        entries: Vec<LogEntry>,
        follower: Option<Receiver<LogEntry>>,
    ) -> Self {
        Self {
            entries,
            view: ViewState { scroll: 0, viewport_height: 1 },
            should_quit: false,
            source: SourceState { name: source_name, follower },
            filters: FilterState::new(),
            ui: UiState { help_visible: false, last_action: None },
        }
    }
    pub(crate) fn apply_startup_filters(&mut self, query: Option<String>, levels: &[LogLevel]) {
        if let Some(query) = query {
            self.filters.set_query(query);
        }
        self.filters.set_levels(levels);
    }
    pub(crate) fn tick(&mut self) {
        self.drain_followed_entries();
        self.clamp_scroll();
    }
    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub(crate) fn draw(&mut self, frame: &mut Frame, area: Rect, shell: ShellContext) {
        self.view.viewport_height = area.height.saturating_sub(7) as usize;
        let panel = Paragraph::new(self.build_lines(shell)).block(
            Block::bordered()
                .title(layout::feature_title(shell.feature_name()))
                .title_style(Style::default().fg(theme::to_ratatui(theme::ACCENT)))
                .border_style(Style::default().fg(theme::to_ratatui(theme::BORDER_STRONG)))
                .style(Style::default().bg(theme::to_ratatui(theme::SURFACE_BASE))),
        );
        frame.render_widget(panel, area);
    }
    pub(crate) fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => {}
        }
    }
    fn build_lines(&self, shell: ShellContext) -> Vec<Line<'static>> {
        if self.ui.help_visible {
            return layout::help_lines(shell.app_name(), shell.feature_name(), Self::EXPORT_PATH);
        }
        let filtered = self.filtered_entries();
        let shown = filtered.len();
        let visible = filtered.into_iter().skip(self.view.scroll).take(self.view.viewport_height);
        let mut lines = Vec::from(layout::feature_header_lines(
            &self.source.name,
            shown,
            self.entries.len(),
            self.view.scroll,
            self.source.follower.is_some(),
        ));
        lines.push(Line::from(self.filters.status_line()));
        lines.push(Line::from(layout::status_line(self.ui.last_action.as_deref())));
        lines.push(Line::from(""));
        if shown == 0 {
            lines.extend(layout::empty_state_lines(!self.entries.is_empty()));
        } else {
            lines.extend(visible.map(LogEntry::render_line));
        }
        lines.extend(layout::footer_lines(shell.feature_name()));
        lines
    }
    fn filtered_entries(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|entry| self.filters.matches(entry)).collect()
    }
    fn filtered_len(&self) -> usize {
        self.entries.iter().filter(|entry| self.filters.matches(entry)).count()
    }
    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.step_down(3),
            MouseEventKind::ScrollUp => self.view.scroll = self.view.scroll.saturating_sub(3),
            _ => {}
        }
    }
    fn handle_key(&mut self, key: KeyEvent) {
        if self.ui.help_visible {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                self.ui.help_visible = false;
            }
            return;
        }
        if self.filters.handle_key(key.code) {
            self.clamp_scroll();
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.ui.help_visible = true,
            KeyCode::Char('x') => self.export_filtered(),
            KeyCode::Down | KeyCode::Char('j') => self.step_down(1),
            KeyCode::Up | KeyCode::Char('k') => self.view.scroll = self.view.scroll.saturating_sub(1),
            KeyCode::PageDown => self.step_down(self.view.viewport_height),
            KeyCode::PageUp => self.view.scroll = self.view.scroll.saturating_sub(self.view.viewport_height),
            KeyCode::Home => self.view.scroll = 0,
            KeyCode::End => self.view.scroll = self.max_scroll(),
            _ => {}
        }
    }
    fn export_filtered(&mut self) {
        let filtered = self.filtered_entries();
        self.ui.last_action = Some(match export::write_filtered(Self::EXPORT_PATH, &filtered) {
            Ok(count) => format!("status: exported {count} entries to {}", Self::EXPORT_PATH),
            Err(error) => format!("status: export failed: {error}"),
        });
    }
    fn step_down(&mut self, amount: usize) {
        self.view.scroll = (self.view.scroll + amount).min(self.max_scroll());
    }
    fn max_scroll(&self) -> usize {
        self.filtered_len().saturating_sub(self.view.viewport_height)
    }
    fn clamp_scroll(&mut self) {
        self.view.scroll = self.view.scroll.min(self.max_scroll());
    }
    fn drain_followed_entries(&mut self) {
        loop {
            let next_entry = match self.source.follower.as_mut() {
                Some(receiver) => receiver.try_recv(),
                None => return,
            };
            match next_entry {
                Ok(entry) => self.append_entry(entry),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.source.follower = None;
                    self.ui.last_action = Some("status: live follow stopped".to_string());
                    return;
                }
            }
        }
    }
    fn append_entry(&mut self, entry: LogEntry) {
        let stay_at_bottom = self.view.scroll >= self.max_scroll();
        self.entries.push(entry);
        if stay_at_bottom {
            self.view.scroll = self.max_scroll();
        }
    }
}
