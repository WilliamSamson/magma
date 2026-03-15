use crossterm::event::Event;
use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Margin,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::features::logs::LogsFeature;
use crate::ui::{layout, theme};

pub(crate) struct App {
    logs: LogsFeature,
}

#[derive(Clone, Copy)]
pub(crate) struct ShellContext {
    app_name: &'static str,
    feature_name: &'static str,
}

impl App {
    pub(crate) fn new_logs(logs: LogsFeature) -> Self {
        Self { logs }
    }

    pub(crate) fn render(&mut self, width: u16, height: u16) -> Buffer {
        let shell = self.shell_context();
        let backend = TestBackend::new(width.max(1), height.max(1));
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(_) => return Buffer::empty(Rect::new(0, 0, width.max(1), height.max(1))),
        };
        let completed = match terminal.draw(|frame| self.draw(frame, shell)) {
            Ok(completed) => completed,
            Err(_) => return Buffer::empty(Rect::new(0, 0, width.max(1), height.max(1))),
        };

        completed.buffer.clone()
    }

    pub(crate) fn tick(&mut self) {
        self.logs.tick();
    }

    pub(crate) fn handle_event(&mut self, event: Event) {
        self.logs.handle_event(event);
    }

    pub(crate) fn should_quit(&self) -> bool {
        self.logs.should_quit()
    }

    pub(crate) fn show_dock(&self) -> bool {
        true
    }

    fn shell_context(&self) -> ShellContext {
        ShellContext {
            app_name: "magma",
            feature_name: "logs",
        }
    }

    fn draw(&mut self, frame: &mut Frame, shell: ShellContext) {
        let root = Block::new().style(
            Style::default()
                .bg(theme::to_ratatui(theme::BG_PRIMARY))
                .fg(theme::to_ratatui(theme::TEXT_PRIMARY)),
        );
        frame.render_widget(root, frame.area());

        let [header, body] = layout::workspace_sections(frame.area());
        let header_block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::to_ratatui(theme::BORDER_STRONG)))
            .style(Style::default().bg(theme::to_ratatui(theme::SURFACE_BASE)));
        let header_panel = Paragraph::new(layout::shell_header_lines(shell.app_name(), shell.feature_name()))
            .block(header_block);
        frame.render_widget(header_panel, header);

        let [sidebar, content] = layout::workspace_columns(body);

        let sidebar_block = Block::new()
            .borders(Borders::RIGHT)
            .title(" navigator ")
            .title_style(Style::default().fg(theme::to_ratatui(theme::ACCENT)))
            .border_style(Style::default().fg(theme::to_ratatui(theme::BORDER_STRONG)))
            .style(Style::default()
                .bg(theme::to_ratatui(theme::BG_SIDEBAR))
                .fg(theme::to_ratatui(theme::TEXT_PRIMARY)));
        let sidebar_panel = Paragraph::new(layout::sidebar_lines(shell.app_name(), shell.feature_name()))
            .block(sidebar_block);
        frame.render_widget(sidebar_panel, sidebar);

        let content_block = Block::new()
            .style(Style::default()
                .bg(theme::to_ratatui(theme::BG_SECONDARY))
                .fg(theme::to_ratatui(theme::TEXT_PRIMARY)));
        frame.render_widget(content_block, content);
        self.logs.draw(frame, content.inner(Margin { vertical: 0, horizontal: 1 }), shell);
    }
}

impl ShellContext {
    pub(crate) fn app_name(self) -> &'static str {
        self.app_name
    }

    pub(crate) fn feature_name(self) -> &'static str {
        self.feature_name
    }
}
