use ratatui::{
    layout::{Constraint, Rect},
    text::Span,
    widgets::{Block, Borders, Row, Table, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::format_currency;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme, focused: bool) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" PROJECT COSTS DASHBOARD ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.top_projects.is_empty() {
        frame.render_widget(
            Paragraph::new("No project data available.").style(theme.label_style()),
            inner,
        );
        return;
    }

    let mut rows = Vec::new();

    // Header
    let header_cells = [
        "Project",
        "Messages",
        "Tokens (Est.)",
        "Cost (Est.)",
    ]
    .iter()
    .map(|h| Span::styled(*h, theme.title_style()));
    
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Filter projects that have actual token limits or messages, and sort by cost descending
    let mut sorted_projects = app.top_projects.clone();
    sorted_projects.sort_by(|a, b| b.estimated_cost.partial_cmp(&a.estimated_cost).unwrap_or(std::cmp::Ordering::Equal));

    for project in sorted_projects {
        let row = Row::new(vec![
            Span::styled(
                format!("{:<30}", truncate_str(&project.name, 30)),
                theme.value_style(),
            ),
            Span::styled(format_number(project.message_count), theme.value_style()),
            Span::styled(format_number(project.estimated_tokens), theme.value_style()),
            Span::styled(format_currency(project.estimated_cost), theme.warning_style()),
        ]);
        rows.push(row);
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(32),
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .column_spacing(2);

    frame.render_widget(table, inner);
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
