use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ui::theme;

/// Split the workspace into a compact shell header and the main body.
pub(crate) fn workspace_sections(area: Rect) -> [Rect; 2] {
    Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area)
}

/// Split the workspace into sidebar (20%) and content (80%).
pub(crate) fn workspace_columns(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]).areas(area)
}

pub(crate) fn feature_title(feature_name: &str) -> String {
    format!(" {} ", feature_name.to_ascii_uppercase())
}

pub(crate) fn shell_header_lines(app_name: &str, feature_name: &str) -> Vec<Line<'static>> {
    let primary = theme::to_ratatui(theme::TEXT_PRIMARY);
    let secondary = theme::to_ratatui(theme::TEXT_SECONDARY);
    vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", app_name.to_ascii_uppercase()),
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} workspace", feature_name.to_ascii_uppercase()),
                Style::default().fg(secondary),
            ),
            Span::styled("  |  ", Style::default().fg(theme::to_ratatui(theme::TEXT_DIM))),
            Span::styled("q quit", Style::default().fg(secondary)),
            Span::styled("  / search", Style::default().fg(secondary)),
            Span::styled("  ? help", Style::default().fg(secondary)),
        ]),
    ]
}

/// Render the sidebar content lines with accent-colored sections.
pub(crate) fn sidebar_lines(app_name: &str, active_feature: &str) -> Vec<Line<'static>> {
    let accent = theme::to_ratatui(theme::ACCENT);
    let dim = theme::to_ratatui(theme::TEXT_DIM);
    let secondary = theme::to_ratatui(theme::TEXT_SECONDARY);
    let primary = theme::to_ratatui(theme::TEXT_PRIMARY);
    let warning = theme::to_ratatui(theme::WARNING);

    let active_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
    let dim_style = Style::default().fg(dim);
    let secondary_style = Style::default().fg(secondary);
    let warning_style = Style::default().fg(warning).add_modifier(Modifier::BOLD);

    // Active feature indicator uses a left bar character.
    let terminal_line = if active_feature == "terminal" {
        Line::from(vec![
            Span::styled("  ▎ ", active_style),
            Span::styled("terminal", active_style),
        ])
    } else {
        Line::from(Span::styled("    terminal", secondary_style))
    };

    let logs_line = if active_feature == "logs" {
        Line::from(vec![
            Span::styled("  ▎ ", active_style),
            Span::styled("logs", active_style),
        ])
    } else {
        Line::from(Span::styled("    logs", secondary_style))
    };

    vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", app_name.to_ascii_uppercase()),
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("  desktop console", secondary_style)),
        Line::from(""),
        Line::from(Span::styled("  CONTROL SURFACE", dim_style)),
        terminal_line,
        logs_line,
        Line::from(""),
        Line::from(Span::styled("  SESSION", dim_style)),
        Line::from(Span::styled(
            format!("    mode {active_feature}"),
            warning_style,
        )),
        Line::from(""),
        Line::from(Span::styled("  COMMANDS", dim_style)),
        Line::from(Span::styled("    q quit", secondary_style)),
        Line::from(Span::styled("    ? help", secondary_style)),
        Line::from(Span::styled("    / search", secondary_style)),
        Line::from(""),
        Line::from(Span::styled("  LAYOUT", dim_style)),
        Line::from(Span::styled("    left  navigator", secondary_style)),
        Line::from(Span::styled(format!("    right {active_feature}"), secondary_style)),
        Line::from(""),
        Line::from(Span::styled("  PALETTE", dim_style)),
        Line::from(Span::styled("    black white red", secondary_style)),
        Line::from(Span::styled("    green yellow accents", secondary_style)),
        Line::from(""),
        Line::from(Span::styled("  WINDOW", dim_style)),
        Line::from(Span::styled("    matte black chrome", secondary_style)),
        Line::from(Span::styled("    clear white edge", secondary_style)),
    ]
}

pub(crate) fn feature_header_lines(
    source_name: &str,
    shown: usize,
    total: usize,
    scroll: usize,
    live: bool,
) -> [Line<'static>; 2] {
    let secondary = theme::to_ratatui(theme::TEXT_SECONDARY);
    let style = Style::default().fg(secondary);
    [
        Line::from(Span::styled(format!("source: {source_name}"), style)),
        Line::from(Span::styled(
            format!(
                "shown: {shown}/{total}  scroll: {scroll}  mode: {}",
                if live { "live" } else { "static" }
            ),
            style,
        )),
    ]
}

pub(crate) fn status_line(last_action: Option<&str>) -> String {
    last_action.unwrap_or("status: ready").to_string()
}

pub(crate) fn empty_state_lines(has_entries: bool) -> [Line<'static>; 2] {
    let dim = theme::to_ratatui(theme::TEXT_DIM);
    let style = Style::default().fg(dim);
    if has_entries {
        [
            Line::from(Span::styled("No log entries match the current filters.", style)),
            Line::from(Span::styled("Press c to clear filters or / to change the search query.", style)),
        ]
    } else {
        [
            Line::from(Span::styled("Obsidian is open and waiting for log input.", style)),
            Line::from(Span::styled("Pass a file path or pipe newline-delimited JSON into obsidian.", style)),
        ]
    }
}

pub(crate) fn footer_lines(feature_name: &str) -> [Line<'static>; 3] {
    let dim = theme::to_ratatui(theme::TEXT_DIM);
    let style = Style::default().fg(dim);
    [
        Line::from(""),
        Line::from(Span::styled(
            format!("obsidian/{feature_name}  Nav: Up/Down j/k PgUp/PgDn Home/End"),
            style,
        )),
        Line::from(Span::styled(
            "Levels: t=trace d=debug i=info w=warn e=error  Export: x  Quit: q",
            style,
        )),
    ]
}

pub(crate) fn help_lines(app_name: &str, feature_name: &str, export_path: &str) -> Vec<Line<'static>> {
    let primary = theme::to_ratatui(theme::TEXT_PRIMARY);
    let secondary = theme::to_ratatui(theme::TEXT_SECONDARY);
    let accent = theme::to_ratatui(theme::ACCENT);

    vec![
        Line::from(Span::styled(
            format!("{app_name} Help"),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(format!("Current feature: {feature_name}"), Style::default().fg(primary))),
        Line::from(Span::styled("Navigation: Up/Down, j/k, PageUp/PageDown, Home/End", Style::default().fg(secondary))),
        Line::from(Span::styled("Mouse: wheel up/down scrolls the current view", Style::default().fg(secondary))),
        Line::from(Span::styled("Search: / enters search mode, Enter or Esc exits search mode", Style::default().fg(secondary))),
        Line::from(Span::styled("Filter: t d i w e toggle trace/debug/info/warn/error", Style::default().fg(secondary))),
        Line::from(Span::styled("Clear: c resets query and level filters", Style::default().fg(secondary))),
        Line::from(Span::styled(format!("Export: x writes filtered view to {export_path}"), Style::default().fg(secondary))),
        Line::from(Span::styled("Live follow: file inputs append new entries while the UI runs", Style::default().fg(secondary))),
        Line::from(Span::styled("Visual system: black white red with restrained green/yellow accents", Style::default().fg(secondary))),
        Line::from(""),
        Line::from(Span::styled("Press ? or Esc to close this panel.", Style::default().fg(primary))),
    ]
}
