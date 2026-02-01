use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::{Trend, render_sparkline, format_number};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" USAGE TRENDS ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // Today vs Yesterday header
        Constraint::Length(1), // Messages
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // This Week vs Last header
        Constraint::Length(1), // Messages
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // 7-day trend label
        Constraint::Length(1), // Sparkline
        Constraint::Min(0),
    ])
    .split(inner);

    let trend_data = &app.trend_data;

    // Today vs Yesterday
    let line = Line::from(vec![
        Span::styled("── Today vs Yesterday ──", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Day messages comparison
    let day_trend_style = match trend_data.day.messages_trend {
        Trend::Up => theme.trend_up_style(),
        Trend::Down => theme.trend_down_style(),
        _ => theme.trend_flat_style(),
    };

    let change_str = match trend_data.day.messages_change_pct {
        Some(pct) => format!("{} ({:+.0}%)", trend_data.day.messages_trend.symbol(), pct),
        None => trend_data.day.messages_trend.symbol().to_string(),
    };

    let line = Line::from(vec![
        Span::styled("Messages   ", theme.label_style()),
        Span::styled(format!("{:<6}", format_number(trend_data.day.current_messages)), theme.value_style()),
        Span::styled(change_str, day_trend_style),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[1]);

    // This Week vs Last Week
    let line = Line::from(vec![
        Span::styled("── This Week vs Last ──", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[3]);

    // Week messages comparison
    let week_trend_style = match trend_data.week.messages_trend {
        Trend::Up => theme.trend_up_style(),
        Trend::Down => theme.trend_down_style(),
        _ => theme.trend_flat_style(),
    };

    let week_change_str = match trend_data.week.messages_change_pct {
        Some(pct) => format!("{} ({:+.0}%)", trend_data.week.messages_trend.symbol(), pct),
        None => trend_data.week.messages_trend.symbol().to_string(),
    };

    let line = Line::from(vec![
        Span::styled("Messages   ", theme.label_style()),
        Span::styled(format!("{:<6}", format_number(trend_data.week.current_messages)), theme.value_style()),
        Span::styled(week_change_str, week_trend_style),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[4]);

    // 7-day trend sparkline
    let line = Line::from(vec![
        Span::styled("7-day trend:", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[6]);

    let sparkline_str = render_sparkline(&trend_data.sparkline);
    let line = Line::from(vec![
        Span::styled(sparkline_str, theme.sparkline_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[7]);
}
