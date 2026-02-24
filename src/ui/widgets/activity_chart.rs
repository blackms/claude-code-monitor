use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme::Theme;

const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" DAILY ACTIVITY (Last 14 Days) ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else {
        frame.render_widget(Paragraph::new("No data available"), inner);
        return;
    };

    // Get last 14 days of activity
    let activities: Vec<_> = stats
        .daily_activity
        .iter()
        .rev()
        .take(14)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if activities.is_empty() {
        frame.render_widget(Paragraph::new("No activity data"), inner);
        return;
    }

    let values: Vec<u64> = activities.iter().map(|a| a.message_count).collect();
    let max_value = values.iter().max().copied().unwrap_or(1);
    let min_value = values.iter().min().copied().unwrap_or(0);
    let range = (max_value - min_value).max(1);

    // Create sparkline
    let mut sparkline = String::new();
    for &value in &values {
        let normalized = ((value - min_value) as f64 / range as f64 * 7.0).round() as usize;
        sparkline.push(SPARKLINE_CHARS[normalized.min(7)]);
        sparkline.push(' ');
    }

    // Create date labels
    let dates: Vec<String> = activities
        .iter()
        .map(|a| {
            let parts: Vec<&str> = a.date.split('-').collect();
            if parts.len() >= 3 {
                format!("{}/{}", parts[1], parts[2])
            } else {
                a.date.clone()
            }
        })
        .collect();

    let line = Line::from(vec![Span::styled(sparkline, theme.highlight_style())]);
    frame.render_widget(Paragraph::new(line), inner);

    // Show dates if there's room
    if inner.height > 1 {
        let date_labels: String = dates
            .iter()
            .map(|d| format!("{:<2}", &d[3..]))
            .collect::<Vec<_>>()
            .join("");

        let date_line = Line::from(vec![Span::styled(date_labels, theme.label_style())]);
        let date_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(date_line), date_area);
    }
}
