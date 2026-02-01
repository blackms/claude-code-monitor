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
        Constraint::Min(0),
    ])
    .split(inner);

    // Session stats
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

    let today_messages = app
        .stats
        .as_ref()
        .and_then(|s| s.daily_activity.last())
        .map(|d| d.message_count)
        .unwrap_or(0);

    let today_sessions = app
        .stats
        .as_ref()
        .and_then(|s| s.daily_activity.last())
        .map(|d| d.session_count)
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

    // Spacer
    frame.render_widget(Paragraph::new(""), chunks[2]);

    // Today header
    let line = Line::from(vec![Span::styled("── Today ──", theme.label_style())]);
    frame.render_widget(Paragraph::new(line), chunks[3]);

    // Today's messages
    let line = Line::from(vec![
        Span::styled("Messages:    ", theme.label_style()),
        Span::styled(format_number(today_messages), theme.highlight_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[4]);

    // Today's sessions
    let line = Line::from(vec![
        Span::styled("Sessions:    ", theme.label_style()),
        Span::styled(format_number(today_sessions), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[5]);
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
