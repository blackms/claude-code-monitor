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
    let size = frame.size();

    if app.selected_session_id.is_some() {
        render_session_details_layout(frame, app);
        return;
    }

    if size.height < 35 {
        render_compact_layout(frame, app);
    } else if size.height < 50 {
        render_medium_layout(frame, app);
    } else {
        render_full_layout(frame, app);
    }
}

fn render_session_details_layout(frame: &mut Frame, app: &App) {
    let theme = Theme::from_preset(&app.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Session Details
            Constraint::Length(1), // Status bar
        ])
        .split(frame.size());

    render_header(frame, chunks[0], &theme);
    widgets::session_details::render(frame, chunks[1], app, &theme);
    render_status_bar(frame, chunks[2], app, &theme);
}

/// Compact layout (< 35 rows): Summary + Quota + Sessions
fn render_compact_layout(frame: &mut Frame, app: &App) {
    let theme = Theme::from_preset(&app.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Summary + Quota side by side
            Constraint::Length(6), // Sessions
            Constraint::Length(1), // Status bar
        ])
        .split(frame.size());

    render_header(frame, chunks[0], &theme);

    // Split main area horizontally
    let main_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    widgets::summary::render(
        frame,
        main_cols[0],
        app,
        &theme,
        app.focused_panel == Panel::Summary,
    );
    widgets::cost_display::render_usage_quota(
        frame,
        main_cols[1],
        app,
        &theme,
        app.focused_panel == Panel::UsageQuota,
    );

    widgets::session_list::render(
        frame,
        chunks[2],
        app,
        &theme,
        app.focused_panel == Panel::Sessions,
    );
    render_status_bar(frame, chunks[3], app, &theme);
}

/// Medium layout (35-50 rows): Full layout without Hourly chart
fn render_medium_layout(frame: &mut Frame, app: &App) {
    let theme = Theme::from_preset(&app.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(14),   // Main content
            Constraint::Length(8), // Usage quota
            Constraint::Length(3), // Daily activity
            Constraint::Length(5), // Sessions
            Constraint::Length(1), // Status bar
        ])
        .split(frame.size());

    render_header(frame, chunks[0], &theme);
    render_main_content(frame, chunks[1], app, &theme);
    widgets::cost_display::render_usage_quota(
        frame,
        chunks[2],
        app,
        &theme,
        app.focused_panel == Panel::UsageQuota,
    );
    widgets::activity_chart::render(
        frame,
        chunks[3],
        app,
        &theme,
        app.focused_panel == Panel::ActivityChart,
    );
    widgets::session_list::render(
        frame,
        chunks[4],
        app,
        &theme,
        app.focused_panel == Panel::Sessions,
    );
    render_status_bar(frame, chunks[5], app, &theme);
}

/// Full layout (>= 50 rows): Complete layout with Top Projects
fn render_full_layout(frame: &mut Frame, app: &App) {
    let theme = Theme::from_preset(&app.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(16),   // Main content
            Constraint::Length(8), // Usage quota
            Constraint::Length(3), // Daily activity
            Constraint::Length(5), // Hourly distribution
            Constraint::Length(8), // Sessions + Top Projects
            Constraint::Length(1), // Status bar
        ])
        .split(frame.size());

    render_header(frame, chunks[0], &theme);
    render_main_content_full(frame, chunks[1], app, &theme);
    widgets::cost_display::render_usage_quota(
        frame,
        chunks[2],
        app,
        &theme,
        app.focused_panel == Panel::UsageQuota,
    );
    widgets::activity_chart::render(
        frame,
        chunks[3],
        app,
        &theme,
        app.focused_panel == Panel::ActivityChart,
    );
    widgets::hourly_chart::render(
        frame,
        chunks[4],
        app,
        &theme,
        app.focused_panel == Panel::HourlyChart,
    );

    // Sessions + Top Projects side by side
    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[5]);

    widgets::session_list::render(
        frame,
        bottom_cols[0],
        app,
        &theme,
        app.focused_panel == Panel::Sessions,
    );
    render_top_projects_panel(frame, bottom_cols[1], app, &theme);

    render_status_bar(frame, chunks[6], app, &theme);
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let title = Line::from(vec![Span::styled(
        "                    Claude Code Monitor v0.3.0                    ",
        theme.title_style(),
    )]);

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
            Constraint::Percentage(45), // Summary (with extended stats)
            Constraint::Percentage(30), // Cache Efficiency
            Constraint::Percentage(25), // Cost Summary
        ])
        .split(columns[0]);

    widgets::summary::render(
        frame,
        left_chunks[0],
        app,
        theme,
        app.focused_panel == Panel::Summary,
    );
    widgets::statistics::render_cache_efficiency(
        frame,
        left_chunks[1],
        app,
        theme,
        app.focused_panel == Panel::CacheEfficiency,
    );
    widgets::cost_display::render_costs_summary(
        frame,
        left_chunks[2],
        app,
        theme,
        app.focused_panel == Panel::CostsSummary,
    );

    // Right column: Token Usage + Trends + Cost Breakdown
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Token Usage
            Constraint::Percentage(35), // Trends
            Constraint::Percentage(25), // Cost Breakdown
        ])
        .split(columns[1]);

    widgets::token_usage::render(
        frame,
        right_chunks[0],
        app,
        theme,
        app.focused_panel == Panel::TokenUsage,
    );
    widgets::trend_display::render(
        frame,
        right_chunks[1],
        app,
        theme,
        app.focused_panel == Panel::Trends,
    );
    widgets::cost_display::render_cost_breakdown(
        frame,
        right_chunks[2],
        app,
        theme,
        app.focused_panel == Panel::CostBreakdown,
    );
}

/// Full main content layout with all panels
fn render_main_content_full(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // Split into left and right columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left column: Summary + Cache Efficiency + Cost Summary
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Summary
            Constraint::Percentage(30), // Cache Efficiency
            Constraint::Percentage(25), // Cost Summary
        ])
        .split(columns[0]);

    widgets::summary::render(
        frame,
        left_chunks[0],
        app,
        theme,
        app.focused_panel == Panel::Summary,
    );
    widgets::statistics::render_cache_efficiency(
        frame,
        left_chunks[1],
        app,
        theme,
        app.focused_panel == Panel::CacheEfficiency,
    );
    widgets::cost_display::render_costs_summary(
        frame,
        left_chunks[2],
        app,
        theme,
        app.focused_panel == Panel::CostsSummary,
    );

    // Right column: Token Usage + Trends + Cost Breakdown
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35), // Token Usage
            Constraint::Percentage(40), // Trends (more space for month comparison)
            Constraint::Percentage(25), // Cost Breakdown
        ])
        .split(columns[1]);

    widgets::token_usage::render(
        frame,
        right_chunks[0],
        app,
        theme,
        app.focused_panel == Panel::TokenUsage,
    );
    widgets::trend_display::render(
        frame,
        right_chunks[1],
        app,
        theme,
        app.focused_panel == Panel::Trends,
    );
    widgets::cost_display::render_cost_breakdown(
        frame,
        right_chunks[2],
        app,
        theme,
        app.focused_panel == Panel::CostBreakdown,
    );
}

/// Render Top Projects panel
fn render_top_projects_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" TOP PROJECTS ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.top_projects.is_empty() {
        let line = Line::from(vec![Span::styled("No project data", theme.label_style())]);
        frame.render_widget(Paragraph::new(line), inner);
        return;
    }

    let mut constraints: Vec<Constraint> = app
        .top_projects
        .iter()
        .take(5)
        .map(|_| Constraint::Length(1))
        .collect();
    constraints.push(Constraint::Min(0));

    let chunks = Layout::vertical(constraints).split(inner);

    for (i, project) in app.top_projects.iter().take(5).enumerate() {
        let name = truncate_str(&project.name, 18);
        let line = Line::from(vec![
            Span::styled(format!("{:<18}", name), theme.value_style()),
            Span::styled(
                format!(" {:>5}", format_number(project.message_count)),
                theme.label_style(),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i]);
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
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

    let error_display = if let Some(ref msg) = app.export_message {
        Span::styled(format!(" │ {}", msg), theme.success_style())
    } else if let Some(ref err) = app.last_error {
        Span::styled(
            format!(" │ Error: {}", truncate_error(err)),
            theme.warning_style(),
        )
    } else {
        Span::raw("")
    };

    let help = Span::styled(
        " │ q: Quit │ r: Refresh │ e: Export │ Tab: Navigate │ ↑↓: Scroll",
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
