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
        .title(" RECENT SESSIONS ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.sessions.is_empty() {
        frame.render_widget(Paragraph::new("No sessions found"), inner);
        return;
    }

    let visible_count = inner.height as usize;
    let start_idx = app.session_scroll;
    let visible_sessions: Vec<_> = app
        .sessions
        .iter()
        .skip(start_idx)
        .take(visible_count)
        .collect();

    let constraints: Vec<Constraint> = (0..visible_sessions.len())
        .map(|_| Constraint::Length(1))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let chunks = Layout::vertical(constraints).split(inner);

    for (i, session) in visible_sessions.iter().enumerate() {
        let is_selected = start_idx + i == app.selected_session;

        let session_id_short = &session.session_id[..8.min(session.session_id.len())];
        let project_name = truncate_string(&session.project_name, 24);
        let time = session.formatted_time();

        let status = if session.is_active {
            Span::styled("Active", theme.success_style())
        } else {
            Span::styled(
                format!("{} msgs", session.message_count),
                theme.label_style(),
            )
        };

        let prefix = if is_selected && focused {
            Span::styled("▸ ", theme.highlight_style())
        } else {
            Span::styled("  ", theme.label_style())
        };

        let line = Line::from(vec![
            prefix,
            Span::styled(format!("{} ", session_id_short), theme.value_style()),
            Span::styled(format!("{:<24} ", project_name), theme.label_style()),
            Span::styled(format!("{} ", time), theme.label_style()),
            status,
        ]);

        let style = if is_selected && focused {
            ratatui::style::Style::default().bg(ratatui::style::Color::DarkGray)
        } else {
            ratatui::style::Style::default()
        };

        frame.render_widget(Paragraph::new(line).style(style), chunks[i]);
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
