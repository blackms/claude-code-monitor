use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::data::{format_currency, ModelPricing};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" MODEL BREAKDOWN DASHBOARD ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(theme.border_focused_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else {
        frame.render_widget(
            Paragraph::new("No statistics available.").style(theme.label_style()),
            inner,
        );
        return;
    };

    if stats.model_usage.is_empty() {
        frame.render_widget(
            Paragraph::new("No model usage data.").style(theme.label_style()),
            inner,
        );
        return;
    }

    let mut model_stats: Vec<_> = stats.model_usage.iter().collect();
    // Sort by cost descending, then total tokens
    model_stats.sort_by(|a, b| {
        let b_total = b.1.input_tokens
            + b.1.output_tokens
            + b.1.cache_read_input_tokens
            + b.1.cache_creation_input_tokens;
        let a_total = a.1.input_tokens
            + a.1.output_tokens
            + a.1.cache_read_input_tokens
            + a.1.cache_creation_input_tokens;
        b_total.cmp(&a_total)
    });

    let mut rows = Vec::new();
    let mut total_cost = 0.0;

    // Header
    let header_cells = [
        "Model",
        "Input",
        "Output",
        "Cache Read",
        "Cache Create",
        "Cost (Est.)",
    ]
    .iter()
    .map(|h| Span::styled(*h, theme.title_style()));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    for (model, usage) in model_stats {
        let pricing = ModelPricing::for_model(model);
        let cost = (usage.input_tokens as f64 / 1_000_000.0) * pricing.input
            + (usage.output_tokens as f64 / 1_000_000.0) * pricing.output
            + (usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.cache_read
            + (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * pricing.cache_create;

        total_cost += cost;

        let row = Row::new(vec![
            Span::styled(
                format!("{:<25}", truncate_str(model, 25)),
                theme.model_color(model),
            ),
            Span::styled(format_number(usage.input_tokens), theme.value_style()),
            Span::styled(format_number(usage.output_tokens), theme.value_style()),
            Span::styled(
                format_number(usage.cache_read_input_tokens),
                theme.value_style(),
            ),
            Span::styled(
                format_number(usage.cache_creation_input_tokens),
                theme.value_style(),
            ),
            Span::styled(format_currency(cost), theme.warning_style()),
        ]);
        rows.push(row);
    }

    // Total Row
    rows.push(Row::new(vec![Span::raw("")]) /* spacer */);
    rows.push(Row::new(vec![
        Span::styled("TOTAL", theme.title_style()),
        Span::raw(""),
        Span::raw(""),
        Span::raw(""),
        Span::raw(""),
        Span::styled(format_currency(total_cost), theme.warning_style()),
    ]));

    let table = Table::new(
        rows,
        [
            Constraint::Length(28),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .column_spacing(2);

    // Let's divide into Table and a quick summary chart underneath
    let chunks = Layout::vertical([Constraint::Min(5)]).split(inner);

    frame.render_widget(table, chunks[0]);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
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
