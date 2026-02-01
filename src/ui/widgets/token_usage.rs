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
        .title(" TOKEN USAGE BY MODEL ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else {
        frame.render_widget(
            Paragraph::new("No data available"),
            inner,
        );
        return;
    };

    // Calculate total tokens per model
    let mut model_totals: Vec<(String, u64)> = stats
        .model_usage
        .iter()
        .map(|(name, usage)| {
            let total = usage.input_tokens
                + usage.output_tokens
                + usage.cache_read_input_tokens
                + usage.cache_creation_input_tokens;
            (name.clone(), total)
        })
        .collect();

    model_totals.sort_by(|a, b| b.1.cmp(&a.1));

    let total_all: u64 = model_totals.iter().map(|(_, t)| *t).sum();
    if total_all == 0 {
        frame.render_widget(
            Paragraph::new("No token usage data"),
            inner,
        );
        return;
    }

    let constraints: Vec<Constraint> = (0..model_totals.len().min(5))
        .map(|_| Constraint::Length(2))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let chunks = Layout::vertical(constraints).split(inner);

    let bar_width = inner.width.saturating_sub(24) as usize;

    for (i, (model_name, total)) in model_totals.iter().take(5).enumerate() {
        let percentage = (*total as f64 / total_all as f64 * 100.0) as u16;
        let filled = (percentage as usize * bar_width) / 100;

        let display_name = shorten_model_name(model_name);
        let color = theme.model_color(model_name);

        // Create bar
        let bar: String = "█".repeat(filled) + &" ".repeat(bar_width.saturating_sub(filled));

        let line = Line::from(vec![
            Span::styled(format!("{:<14} ", display_name), theme.label_style()),
            Span::styled(bar, ratatui::style::Style::default().fg(color)),
            Span::styled(format!(" {:>3}%", percentage), theme.value_style()),
        ]);

        frame.render_widget(Paragraph::new(line), chunks[i]);
    }
}

fn shorten_model_name(name: &str) -> String {
    if name.contains("opus-4-5") {
        "opus-4-5".to_string()
    } else if name.contains("sonnet-4-5") {
        "sonnet-4-5".to_string()
    } else if name.contains("sonnet-4") {
        "sonnet-4".to_string()
    } else if name.contains("haiku") {
        "haiku".to_string()
    } else {
        name.chars().take(14).collect()
    }
}
