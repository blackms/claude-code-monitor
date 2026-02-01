use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::QuotaInfo;
use crate::ui::theme::Theme;

fn format_currency(value: f64) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1).unwrap_or(&"00");

    let mut result = String::new();
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    format!("${}.{}", result.chars().rev().collect::<String>(), decimal_part)
}

// Model pricing per million tokens (USD) - API pricing
struct ModelPricing {
    input: f64,
    output: f64,
    cache_read: f64,
    cache_create: f64,
}

fn get_pricing(model_name: &str) -> ModelPricing {
    if model_name.contains("opus") {
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_read: 1.50,
            cache_create: 18.75,
        }
    } else if model_name.contains("sonnet") {
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_read: 0.30,
            cache_create: 3.75,
        }
    } else if model_name.contains("haiku") {
        ModelPricing {
            input: 0.25,
            output: 1.25,
            cache_read: 0.025,
            cache_create: 0.3125,
        }
    } else {
        // Default to sonnet pricing
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_read: 0.30,
            cache_create: 3.75,
        }
    }
}

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
        let pricing = get_pricing(model_name);

        breakdown.input += (usage.input_tokens as f64 / 1_000_000.0) * pricing.input;
        breakdown.output += (usage.output_tokens as f64 / 1_000_000.0) * pricing.output;
        breakdown.cache_read +=
            (usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.cache_read;
        breakdown.cache_create +=
            (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * pricing.cache_create;
    }

    breakdown
}

pub fn calculate_today_cost(app: &App) -> f64 {
    let Some(stats) = &app.stats else {
        return 0.0;
    };

    let Some(today) = stats.daily_model_tokens.last() else {
        return 0.0;
    };

    let mut total = 0.0;
    for (model_name, tokens) in &today.tokens_by_model {
        let pricing = get_pricing(model_name);
        // Estimate based on output ratio (typically ~20-30% output)
        let output_ratio = 0.25;
        let cache_ratio = 0.60; // Most tokens are cache reads

        let input_tokens = (*tokens as f64) * (1.0 - output_ratio - cache_ratio) * 0.5;
        let output_tokens = (*tokens as f64) * output_ratio;
        let cache_tokens = (*tokens as f64) * cache_ratio;

        total += (input_tokens / 1_000_000.0) * pricing.input;
        total += (output_tokens / 1_000_000.0) * pricing.output;
        total += (cache_tokens / 1_000_000.0) * pricing.cache_read;
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
        .title(" EQUIVALENT API COST ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let costs = calculate_costs(app);
    let today_cost = calculate_today_cost(app);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    // Note about API pricing
    let line = Line::from(vec![
        Span::styled("(if billed per-token)", theme.label_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // All time cost
    let line = Line::from(vec![
        Span::styled("All Time:  ", theme.label_style()),
        Span::styled(format_currency(costs.total()), theme.highlight_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[1]);

    // Today's cost
    let line = Line::from(vec![
        Span::styled("Today:     ", theme.label_style()),
        Span::styled(format_currency(today_cost), theme.value_style()),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[2]);
}

pub fn render_cost_breakdown(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" API COST BREAKDOWN ")
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
            Span::styled(format!("{:>14}", format_currency(*value)), theme.value_style()),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i]);
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

    let chunks = Layout::vertical([
        Constraint::Length(2), // Session
        Constraint::Length(2), // Week
        Constraint::Length(2), // Sonnet
        Constraint::Min(0),
    ])
    .split(inner);

    // Session usage (5-hour window)
    if let Some(session_pct) = app.quota.session_usage {
        render_quota_bar(
            frame,
            chunks[0],
            "Session (5h)",
            session_pct,
            app.quota.session_resets_at.as_deref(),
            theme,
        );
    }

    // Week usage (7-day window)
    if let Some(week_pct) = app.quota.week_usage {
        render_quota_bar(
            frame,
            chunks[1],
            "Week (all)",
            week_pct,
            app.quota.week_resets_at.as_deref(),
            theme,
        );
    }

    // Sonnet-only usage
    if let Some(sonnet_pct) = app.quota.sonnet_usage {
        render_quota_bar(
            frame,
            chunks[2],
            "Sonnet only",
            sonnet_pct,
            app.quota.sonnet_resets_at.as_deref(),
            theme,
        );
    }
}

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
