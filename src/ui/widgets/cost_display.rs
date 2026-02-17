use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::{ModelPricing, QuotaInfo, format_currency, format_hours};
use crate::ui::theme::Theme;

pub struct CostBreakdown {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_create: f64,
}

impl CostBreakdown {
    pub fn total(&self) -> f64 {
        self.input + self.output + self.cache_read + self.cache_create
    }
}

pub fn calculate_costs(app: &App) -> CostBreakdown {
    let mut breakdown = CostBreakdown {
        input: 0.0,
        output: 0.0,
        cache_read: 0.0,
        cache_create: 0.0,
    };

    let Some(stats) = &app.stats else {
        return breakdown;
    };

    for (model_name, usage) in &stats.model_usage {
        let pricing = ModelPricing::for_model(model_name);

        breakdown.input += (usage.input_tokens as f64 / 1_000_000.0) * pricing.input;
        breakdown.output += (usage.output_tokens as f64 / 1_000_000.0) * pricing.output;
        breakdown.cache_read +=
            (usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.cache_read;
        breakdown.cache_create +=
            (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * pricing.cache_create;
    }

    breakdown
}

#[allow(dead_code)]
pub fn calculate_today_cost(app: &App) -> f64 {
    calculate_period_cost(app, 1)
}

/// Returns (cost, label with date) for the last day in stats
pub fn calculate_last_day_cost(app: &App) -> (f64, String) {
    let Some(stats) = &app.stats else {
        return (0.0, "Last day:".to_string());
    };

    let Some(last_day) = stats.daily_model_tokens.last() else {
        return (0.0, "Last day:".to_string());
    };

    // Format the date nicely (e.g., "Jan 31:")
    let label = if let Ok(date) = chrono::NaiveDate::parse_from_str(&last_day.date, "%Y-%m-%d") {
        format!("{}:", date.format("%b %d"))
    } else {
        format!("{}:", &last_day.date)
    };

    let cost = calculate_period_cost(app, 1);
    (cost, label)
}

pub fn calculate_week_cost(app: &App) -> f64 {
    calculate_period_cost(app, 7)
}

pub fn calculate_month_cost(app: &App) -> f64 {
    calculate_period_cost(app, 30)
}

/// Token breakdown ratios for cost estimation
/// daily_model_tokens contains only (input + output), not cache tokens.
/// We calculate:
/// - input_ratio: fraction of (input+output) that is input
/// - cache_read_factor: ratio of cache_read to (input+output) for scaling
/// - cache_create_factor: ratio of cache_create to (input+output) for scaling
struct TokenRatios {
    input_ratio: f64,        // input / (input + output)
    cache_read_factor: f64,  // cache_read / (input + output)
    cache_create_factor: f64, // cache_create / (input + output)
}

fn calculate_token_ratios(stats: &crate::data::StatsCache) -> TokenRatios {
    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let mut total_cache_read = 0u64;
    let mut total_cache_create = 0u64;

    for (_model_name, usage) in &stats.model_usage {
        total_input += usage.input_tokens;
        total_output += usage.output_tokens;
        total_cache_read += usage.cache_read_input_tokens;
        total_cache_create += usage.cache_creation_input_tokens;
    }

    let io_total = (total_input + total_output) as f64;
    if io_total < 1.0 {
        // Fallback to reasonable defaults if no data
        return TokenRatios {
            input_ratio: 0.73,       // typical input ratio
            cache_read_factor: 50.0, // cache is typically much larger
            cache_create_factor: 3.0,
        };
    }

    TokenRatios {
        input_ratio: total_input as f64 / io_total,
        cache_read_factor: total_cache_read as f64 / io_total,
        cache_create_factor: total_cache_create as f64 / io_total,
    }
}

fn calculate_period_cost(app: &App, days: usize) -> f64 {
    let Some(stats) = &app.stats else {
        return 0.0;
    };

    // daily_model_tokens contains (input + output) tokens only, NOT cache.
    // We estimate cache usage proportionally based on all-time ratios.
    let ratios = calculate_token_ratios(stats);

    let mut total = 0.0;
    let recent_days: Vec<_> = stats.daily_model_tokens.iter().rev().take(days).collect();

    for day in recent_days {
        for (model_name, tokens) in &day.tokens_by_model {
            let pricing = ModelPricing::for_model(model_name);
            // tokens = input + output for this day
            let io_tokens = *tokens as f64;

            // Split input+output based on all-time ratio
            let input_tokens = io_tokens * ratios.input_ratio;
            let output_tokens = io_tokens * (1.0 - ratios.input_ratio);

            // Estimate cache usage proportionally
            let cache_read_tokens = io_tokens * ratios.cache_read_factor;
            let cache_create_tokens = io_tokens * ratios.cache_create_factor;

            total += (input_tokens / 1_000_000.0) * pricing.input;
            total += (output_tokens / 1_000_000.0) * pricing.output;
            total += (cache_read_tokens / 1_000_000.0) * pricing.cache_read;
            total += (cache_create_tokens / 1_000_000.0) * pricing.cache_create;
        }
    }

    total
}

pub fn render_costs_summary(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" COST BREAKDOWN (all time) ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let costs = calculate_costs(app);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    let items = [
        ("Input:", costs.input),
        ("Output:", costs.output),
        ("Cache Read:", costs.cache_read),
        ("Cache Create:", costs.cache_create),
    ];

    for (i, (label, value)) in items.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(format!("{:<14}", label), theme.label_style()),
            Span::styled(format!("{:>10}", format_currency(*value)), theme.value_style()),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i]);
    }
}

pub fn render_cost_breakdown(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" API COST BY PERIOD ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (last_day_cost, last_day_label) = calculate_last_day_cost(app);
    let week_cost = calculate_week_cost(app);
    let month_cost = calculate_month_cost(app);
    let all_time_cost = calculate_costs(app).total();

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    // Note about estimates
    let line = Line::from(vec![
        Span::styled("(estimated from daily tokens)", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    let items = [
        (last_day_label, last_day_cost, theme.highlight_style()),
        ("Last 7 days:".to_string(), week_cost, theme.value_style()),
        ("Last 30 days:".to_string(), month_cost, theme.value_style()),
        ("Projected/mo:".to_string(), app.monthly_projection, theme.warning_style()),
        ("All Time:".to_string(), all_time_cost, theme.value_style()),
    ];

    for (i, (label, value, style)) in items.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(format!("{:<14}", label), theme.label_style()),
            Span::styled(format!("{:>14}", format_currency(*value)), *style),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i + 1]);
    }
}

pub fn render_usage_quota(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let title = match &app.quota.subscription_type {
        Some(sub) => format!(" USAGE QUOTA ({}) ", sub),
        None => " USAGE QUOTA ".to_string(),
    };

    let block = Block::default()
        .title(title)
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Check if we have quota data
    if app.quota.session_usage.is_none() && app.quota.week_usage.is_none() {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

        if let Some(ref err) = app.quota.last_error {
            let line = Line::from(vec![
                Span::styled(format!("Error: {}", truncate_str(err, 30)), theme.warning_style()),
            ]);
            frame.render_widget(Paragraph::new(line), chunks[0]);
        } else {
            let line = Line::from(vec![
                Span::styled("Loading quota data...", theme.label_style()),
            ]);
            frame.render_widget(Paragraph::new(line), chunks[0]);
        }
        return;
    }

    // Determine if we need extra usage section
    let has_extra = app.quota.extra_usage_enabled;

    let chunks = if has_extra {
        Layout::vertical([
            Constraint::Length(3), // Session with depletion
            Constraint::Length(3), // Week with depletion
            Constraint::Length(2), // Extra usage
            Constraint::Min(0),
        ])
        .split(inner)
    } else {
        Layout::vertical([
            Constraint::Length(3), // Session with depletion
            Constraint::Length(3), // Week with depletion
            Constraint::Min(0),
        ])
        .split(inner)
    };

    // Session usage (5-hour window)
    if let Some(session_pct) = app.quota.session_usage {
        render_enhanced_quota_bar(
            frame,
            chunks[0],
            "Session (5h)",
            session_pct,
            app.quota.session_resets_at.as_deref(),
            app.quota.session_rate_per_hour,
            app.quota.session_hours_to_depletion,
            app.quota.session_is_safe,
            theme,
        );
    }

    // Week usage (7-day window)
    if let Some(week_pct) = app.quota.week_usage {
        render_enhanced_quota_bar(
            frame,
            chunks[1],
            "Week (all)",
            week_pct,
            app.quota.week_resets_at.as_deref(),
            app.quota.week_rate_per_hour,
            app.quota.week_hours_to_depletion,
            app.quota.week_is_safe,
            theme,
        );
    }

    // Extra usage (pay-as-you-go overage)
    if has_extra && chunks.len() > 2 {
        render_extra_usage(frame, chunks[2], app, theme);
    }
}

fn render_enhanced_quota_bar(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    percentage: f64,
    resets_at: Option<&str>,
    rate_per_hour: Option<f64>,
    hours_to_depletion: Option<f64>,
    is_safe: Option<bool>,
    theme: &Theme,
) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // Label + percentage + rate
        Constraint::Length(1), // Progress bar + reset time
        Constraint::Length(1), // Depletion status
    ])
    .split(area);

    // Label and percentage with rate
    let pct_color = if percentage >= 90.0 {
        theme.error
    } else if percentage >= 70.0 {
        theme.warning
    } else {
        theme.success
    };

    let rate_str = rate_per_hour
        .map(|r| format!(" ({:.1}%/hr)", r))
        .unwrap_or_default();

    let line = Line::from(vec![
        Span::styled(format!("{:<12}", label), theme.label_style()),
        Span::styled(format!("{:>3.0}%", percentage), Style::default().fg(pct_color)),
        Span::styled(rate_str, theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Progress bar using gauge
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(pct_color))
        .ratio((percentage / 100.0).min(1.0))
        .label("");

    // Use a smaller area for the gauge bar
    let bar_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width.saturating_sub(14),
        height: 1,
    };
    frame.render_widget(gauge, bar_area);

    // Reset time
    if let Some(reset) = resets_at {
        let reset_str = QuotaInfo::format_reset_time(reset);
        let reset_area = Rect {
            x: bar_area.x + bar_area.width + 1,
            y: chunks[1].y,
            width: 13,
            height: 1,
        };
        let line = Line::from(vec![
            Span::styled(format!("↻{}", reset_str), theme.label_style()),
        ]);
        frame.render_widget(Paragraph::new(line), reset_area);
    }

    // Depletion status line
    if let Some(safe) = is_safe {
        let (status_text, status_style) = if safe {
            let time_str = hours_to_depletion
                .map(|h| format!("Safe (resets in {}) ✓", format_hours(h)))
                .unwrap_or_else(|| "Safe ✓".to_string());
            (time_str, theme.safe_style())
        } else {
            let time_str = hours_to_depletion
                .map(|h| format!("Depletes in {} ⚠", format_hours(h)))
                .unwrap_or_else(|| "Depleting ⚠".to_string());
            (time_str, theme.danger_style())
        };

        let line = Line::from(vec![
            Span::styled(status_text, status_style),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[2]);
    }
}

fn render_extra_usage(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // Extra usage label + amount
        Constraint::Length(1), // Progress bar or limit info
    ])
    .split(area);

    let used = app.quota.extra_usage_used.unwrap_or(0.0);
    let limit = app.quota.extra_usage_monthly_limit;
    let utilization = app.quota.extra_usage_utilization.unwrap_or(0.0);

    // First line: label and used amount
    let used_style = if used > 0.0 {
        theme.warning_style()
    } else {
        theme.value_style()
    };

    let line = Line::from(vec![
        Span::styled("Extra Usage: ", theme.label_style()),
        Span::styled(format_currency(used), used_style),
        Span::styled(
            limit.map(|l| format!(" / {}", format_currency(l))).unwrap_or_default(),
            theme.label_style(),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Second line: progress bar if limit is set, or just utilization
    if limit.is_some() {
        let pct_color = if utilization >= 90.0 {
            theme.error
        } else if utilization >= 70.0 {
            theme.warning
        } else {
            theme.success
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(pct_color))
            .ratio((utilization / 100.0).min(1.0))
            .label("");

        let bar_area = Rect {
            x: chunks[1].x,
            y: chunks[1].y,
            width: chunks[1].width.saturating_sub(8),
            height: 1,
        };
        frame.render_widget(gauge, bar_area);

        // Percentage
        let pct_area = Rect {
            x: bar_area.x + bar_area.width + 1,
            y: chunks[1].y,
            width: 7,
            height: 1,
        };
        let line = Line::from(vec![
            Span::styled(format!("{:.0}%", utilization), Style::default().fg(pct_color)),
        ]);
        frame.render_widget(Paragraph::new(line), pct_area);
    } else if used > 0.0 {
        let line = Line::from(vec![
            Span::styled("(no monthly limit set)", theme.label_style()),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[1]);
    }
}

#[allow(dead_code)]
fn render_quota_bar(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    percentage: f64,
    resets_at: Option<&str>,
    theme: &Theme,
) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    // Label and percentage
    let pct_color = if percentage >= 90.0 {
        theme.error
    } else if percentage >= 70.0 {
        theme.warning
    } else {
        theme.success
    };

    let line = Line::from(vec![
        Span::styled(format!("{:<12}", label), theme.label_style()),
        Span::styled(format!("{:>3.0}%", percentage), Style::default().fg(pct_color)),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Progress bar using gauge
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(pct_color))
        .ratio((percentage / 100.0).min(1.0))
        .label("");

    // Use a smaller area for the gauge bar
    let bar_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width.saturating_sub(12),
        height: 1,
    };
    frame.render_widget(gauge, bar_area);

    // Reset time
    if let Some(reset) = resets_at {
        let reset_str = QuotaInfo::format_reset_time(reset);
        let reset_area = Rect {
            x: bar_area.x + bar_area.width + 1,
            y: chunks[1].y,
            width: 11,
            height: 1,
        };
        let line = Line::from(vec![
            Span::styled(format!("↻{}", reset_str), theme.label_style()),
        ]);
        frame.render_widget(Paragraph::new(line), reset_area);
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
