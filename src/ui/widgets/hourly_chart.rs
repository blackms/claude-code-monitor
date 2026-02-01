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
        .title(" HOURLY DISTRIBUTION ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else {
        frame.render_widget(Paragraph::new("No data available"), inner);
        return;
    };

    // Convert hour_counts to sorted vec
    let mut hour_data: Vec<(u8, u64)> = stats
        .hour_counts
        .iter()
        .filter_map(|(k, v)| k.parse::<u8>().ok().map(|h| (h, *v)))
        .collect();

    hour_data.sort_by_key(|(h, _)| *h);

    if hour_data.is_empty() {
        frame.render_widget(Paragraph::new("No hourly data"), inner);
        return;
    }

    let max_count = hour_data.iter().map(|(_, c)| *c).max().unwrap_or(1);

    // Display in two columns
    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let bar_width = (chunks[0].width.saturating_sub(10)) as usize;

    // First column: hours 0-11
    let first_half: Vec<_> = hour_data.iter().filter(|(h, _)| *h < 12).collect();
    // Second column: hours 12-23
    let second_half: Vec<_> = hour_data.iter().filter(|(h, _)| *h >= 12).collect();

    render_hour_column(frame, chunks[0], &first_half, max_count, bar_width, theme);
    render_hour_column(frame, chunks[1], &second_half, max_count, bar_width, theme);
}

fn render_hour_column(
    frame: &mut Frame,
    area: Rect,
    hours: &[&(u8, u64)],
    max_count: u64,
    bar_width: usize,
    theme: &Theme,
) {
    let constraints: Vec<Constraint> = (0..hours.len().min(area.height as usize))
        .map(|_| Constraint::Length(1))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let chunks = Layout::vertical(constraints).split(area);

    for (i, (hour, count)) in hours.iter().enumerate() {
        if i >= chunks.len() - 1 {
            break;
        }

        let filled = ((*count as f64 / max_count as f64) * bar_width as f64) as usize;
        let bar: String = "█".repeat(filled);

        let line = Line::from(vec![
            Span::styled(format!("{:02} ", hour), theme.label_style()),
            Span::styled(bar, theme.highlight_style()),
            Span::styled(format!(" {}", count), theme.value_style()),
        ]);

        frame.render_widget(Paragraph::new(line), chunks[i]);
    }
}
