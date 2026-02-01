use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::{format_number, format_first_session_date, Trend};
use crate::ui::theme::Theme;

/// Format tokens in short form (e.g., 12.5K)
fn format_tokens_short(tokens: f64) -> String {
    if tokens >= 1_000_000.0 {
        format!("{:.1}M", tokens / 1_000_000.0)
    } else if tokens >= 1_000.0 {
        format!("{:.1}K", tokens / 1_000.0)
    } else {
        format!("{:.0}", tokens)
    }
}

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
        Constraint::Length(1), // Sessions
        Constraint::Length(1), // Messages
        Constraint::Length(1), // Since date
        Constraint::Length(1), // Web Searches
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Averages header
        Constraint::Length(1), // Avg msgs/session
        Constraint::Length(1), // Avg tokens/msg
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Live header
        Constraint::Length(1), // Today with trend
        Constraint::Length(1), // Last 5h
        Constraint::Length(1), // Active
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

    // First session date (Since)
    let since_str = app
        .stats
        .as_ref()
        .and_then(|s| s.first_session_date.as_ref())
        .map(|d| format_first_session_date(d))
        .unwrap_or_else(|| "--".to_string());
    let line = Line::from(vec![
        Span::styled("Since:       ", theme.label_style()),
        Span::styled(since_str, theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[2]);

    // Web Searches
    let line = Line::from(vec![
        Span::styled("Web Searches:", theme.label_style()),
        Span::styled(format!(" {}", format_number(app.web_search_stats.total_searches)), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[3]);

    // Averages header
    let line = Line::from(vec![
        Span::styled("── Averages ──", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[5]);

    // Avg msgs/session
    let avg_msgs = if app.averages.messages_per_session > 0.0 {
        format!("{:.1}", app.averages.messages_per_session)
    } else {
        "--".to_string()
    };
    let line = Line::from(vec![
        Span::styled("Msgs/session:", theme.label_style()),
        Span::styled(format!(" {}", avg_msgs), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[6]);

    // Avg tokens/msg
    let avg_tokens = if app.averages.tokens_per_message > 0.0 {
        format_tokens_short(app.averages.tokens_per_message)
    } else {
        "--".to_string()
    };
    let line = Line::from(vec![
        Span::styled("Tokens/msg:  ", theme.label_style()),
        Span::styled(format!(" {}", avg_tokens), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[7]);

    // Live header
    let line = Line::from(vec![
        Span::styled("── Live ──", theme.success_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[9]);

    // Today's messages with trend indicator
    let trend_style = match app.trend_data.day.messages_trend {
        Trend::Up => theme.trend_up_style(),
        Trend::Down => theme.trend_down_style(),
        _ => theme.trend_flat_style(),
    };

    let trend_str = match app.trend_data.day.messages_change_pct {
        Some(pct) => format!(" {}({:+.0}%)", app.trend_data.day.messages_trend.symbol(), pct),
        None => String::new(),
    };

    let line = Line::from(vec![
        Span::styled("Today:       ", theme.label_style()),
        Span::styled(format!("{}", format_number(app.today_messages_live)), theme.highlight_style()),
        Span::styled(trend_str, trend_style),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[10]);

    // Last 5 hours messages (LIVE)
    let line = Line::from(vec![
        Span::styled("Last 5h:     ", theme.label_style()),
        Span::styled(format!("{}", format_number(app.recent_5h_messages)), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[11]);

    // Active sessions count
    let active_sessions = app.sessions.iter().filter(|s| s.is_active).count();
    let line = Line::from(vec![
        Span::styled("Active:      ", theme.label_style()),
        Span::styled(format!("{} sessions", active_sessions), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[12]);
}
