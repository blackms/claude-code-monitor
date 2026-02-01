use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" SUMMARY ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    // Session stats (from stats-cache.json - historical)
    let total_sessions = app
        .stats
        .as_ref()
        .map(|s| s.total_sessions)
        .unwrap_or(0);
    let total_messages = app
        .stats
        .as_ref()
        .map(|s| s.total_messages)
        .unwrap_or(0);

    // Total Sessions
    let line = Line::from(vec![
        Span::styled("Sessions:    ", theme.label_style()),
        Span::styled(format_number(total_sessions), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Total Messages
    let line = Line::from(vec![
        Span::styled("Messages:    ", theme.label_style()),
        Span::styled(format_number(total_messages), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[1]);

    // Stats last updated indicator
    let updated_str = app.stats_last_updated.as_deref().unwrap_or("--");
    let line = Line::from(vec![
        Span::styled("Updated:     ", theme.label_style()),
        Span::styled(updated_str, theme.warning_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[2]);

    // Spacer
    frame.render_widget(Paragraph::new(""), chunks[3]);

    // Today header - LIVE from history.jsonl
    let line = Line::from(vec![
        Span::styled("── Live ──", theme.success_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[4]);

    // Today's messages (LIVE from history.jsonl)
    let line = Line::from(vec![
        Span::styled("Today:       ", theme.label_style()),
        Span::styled(format!("{} msgs", format_number(app.today_messages_live)), theme.highlight_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[5]);

    // Last 5 hours messages (LIVE)
    let line = Line::from(vec![
        Span::styled("Last 5h:     ", theme.label_style()),
        Span::styled(format!("{} msgs", format_number(app.recent_5h_messages)), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[6]);

    // Active sessions count
    let active_sessions = app.sessions.iter().filter(|s| s.is_active).count();
    let line = Line::from(vec![
        Span::styled("Active:      ", theme.label_style()),
        Span::styled(format!("{} sessions", active_sessions), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[7]);
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
