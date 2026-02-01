use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Panel};
use crate::ui::theme::Theme;
use crate::ui::widgets;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = Theme::default();

    // Main layout - adjusted for more content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(16),    // Main content (increased)
            Constraint::Length(8),  // Usage quota (increased for depletion info)
            Constraint::Length(3),  // Daily activity
            Constraint::Length(5),  // Hourly distribution
            Constraint::Length(5),  // Sessions
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.size());

    render_header(frame, chunks[0], &theme);
    render_main_content(frame, chunks[1], app, &theme);
    widgets::cost_display::render_usage_quota(frame, chunks[2], app, &theme, app.focused_panel == Panel::UsageQuota);
    widgets::activity_chart::render(frame, chunks[3], app, &theme, app.focused_panel == Panel::ActivityChart);
    widgets::hourly_chart::render(frame, chunks[4], app, &theme, app.focused_panel == Panel::HourlyChart);
    widgets::session_list::render(frame, chunks[5], app, &theme, app.focused_panel == Panel::Sessions);
    render_status_bar(frame, chunks[6], app, &theme);
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let title = Line::from(vec![
        Span::styled(
            "                    Claude Code Monitor v0.2.0                    ",
            theme.title_style(),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let paragraph = Paragraph::new(title).block(block).centered();
    frame.render_widget(paragraph, area);
}

fn render_main_content(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // Split into left and right columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left column: Summary + Cache Efficiency + Cost Breakdown
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),  // Summary (with extended stats)
            Constraint::Percentage(30),  // Cache Efficiency
            Constraint::Percentage(25),  // Cost Summary
        ])
        .split(columns[0]);

    widgets::summary::render(frame, left_chunks[0], app, theme, app.focused_panel == Panel::Summary);
    widgets::statistics::render_cache_efficiency(frame, left_chunks[1], app, theme, app.focused_panel == Panel::CacheEfficiency);
    widgets::cost_display::render_costs_summary(frame, left_chunks[2], app, theme, app.focused_panel == Panel::CostsSummary);

    // Right column: Token Usage + Trends + Cost Breakdown
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Token Usage
            Constraint::Percentage(35),  // Trends
            Constraint::Percentage(25),  // Cost Breakdown
        ])
        .split(columns[1]);

    widgets::token_usage::render(frame, right_chunks[0], app, theme, app.focused_panel == Panel::TokenUsage);
    widgets::trend_display::render(frame, right_chunks[1], app, theme, app.focused_panel == Panel::Trends);
    widgets::cost_display::render_cost_breakdown(frame, right_chunks[2], app, theme, app.focused_panel == Panel::CostBreakdown);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let status_indicator = if app.is_live {
        Span::styled("◉ Live", theme.success_style())
    } else {
        Span::styled("○ Paused", theme.warning_style())
    };

    // Show tracker sample count for debugging
    let tracker_info = if app.usage_tracker.has_enough_data() {
        Span::styled(
            format!(" │ Samples: {}", app.usage_tracker.sample_count()),
            theme.label_style(),
        )
    } else {
        Span::styled(
            format!(" │ Collecting: {}/10", app.usage_tracker.sample_count()),
            theme.label_style(),
        )
    };

    let error_display = if let Some(ref err) = app.last_error {
        Span::styled(format!(" │ Error: {}", truncate_error(err)), theme.warning_style())
    } else {
        Span::raw("")
    };

    let help = Span::styled(
        " │ q: Quit │ r: Refresh │ Tab: Navigate │ ↑↓: Scroll",
        theme.label_style(),
    );

    let line = Line::from(vec![
        Span::raw(" "),
        status_indicator,
        tracker_info,
        error_display,
        help,
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn truncate_error(err: &str) -> String {
    if err.len() > 50 {
        format!("{}…", &err[..49])
    } else {
        err.to_string()
    }
}
