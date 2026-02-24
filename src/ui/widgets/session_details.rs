use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let session_id = match &app.selected_session_id {
        Some(id) => id,
        None => return,
    };

    let history = app.get_session_history(session_id);

    let block = Block::default()
        .title(format!(" Session Details: {} ", session_id))
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(theme.border_focused_style());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if history.is_empty() {
        let msg = Paragraph::new("No history found for this session.").style(theme.label_style());
        frame.render_widget(msg, inner_area);
        return;
    }

    let mut lines = Vec::new();

    for entry in history.iter() {
        let mut timestamp = chrono::Local::now();
        if entry.timestamp > 0 {
            use chrono::TimeZone;
            if let Some(dt) = chrono::Local
                .timestamp_millis_opt(entry.timestamp as i64)
                .single()
            {
                timestamp = dt;
            }
        }

        let time_str = timestamp.format("%b %d %H:%M:%S").to_string();

        lines.push(Line::from(vec![
            Span::styled(format!("[{}] ", time_str), theme.label_style()),
            Span::styled(&entry.display, theme.value_style()),
        ]));

        if !entry.pasted_contents.is_empty() {
            let files: Vec<_> = entry.pasted_contents.keys().collect();
            let summary = format!(
                " (+ {} attachments: {})",
                files.len(),
                files
                    .into_iter()
                    .take(3)
                    .map(|s| {
                        if s.len() > 20 {
                            format!("{}…", &s[..19])
                        } else {
                            s.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(summary, theme.border_style()),
            ]));
        }

        lines.push(Line::raw(""));
    }

    let scroll_offset = app.session_details_scroll as u16;
    let max_scroll = lines.len().saturating_sub(inner_area.height as usize) as u16;
    let actual_scroll = scroll_offset.min(max_scroll);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((actual_scroll, 0));

    frame.render_widget(paragraph, inner_area);
}
