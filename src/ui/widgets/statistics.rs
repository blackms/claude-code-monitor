use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::{format_currency, format_number};
use crate::ui::theme::Theme;

/// Render cache efficiency panel
pub fn render_cache_efficiency(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &Theme,
    focused: bool,
) {
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(" CACHE EFFICIENCY ")
        .title_style(theme.title_style())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let efficiency = &app.cache_efficiency;

    let chunks = Layout::vertical([
        Constraint::Length(1), // Hit ratio
        Constraint::Length(1), // Savings
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // By Model header
        Constraint::Length(1), // Model 1
        Constraint::Length(1), // Model 2
        Constraint::Length(1), // Model 3
        Constraint::Min(0),
    ])
    .split(inner);

    // Hit Ratio
    let ratio_style = if efficiency.hit_ratio >= 80.0 {
        theme.success_style()
    } else if efficiency.hit_ratio >= 50.0 {
        theme.warning_style()
    } else {
        theme.value_style()
    };

    let line = Line::from(vec![
        Span::styled("Hit Ratio:   ", theme.label_style()),
        Span::styled(format!("{:.1}%", efficiency.hit_ratio), ratio_style),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Savings
    let line = Line::from(vec![
        Span::styled("Savings:     ", theme.label_style()),
        Span::styled(
            format_currency(efficiency.savings_usd),
            theme.success_style(),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[1]);

    // By Model header
    let line = Line::from(vec![Span::styled("By Model:", theme.label_style())]);
    frame.render_widget(Paragraph::new(line), chunks[3]);

    // Per-model ratios (up to 3)
    let mut models: Vec<_> = efficiency.per_model.iter().collect();
    models.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (i, (model, ratio)) in models.iter().take(3).enumerate() {
        let line = Line::from(vec![
            Span::styled(format!("  {:<12}", model), theme.label_style()),
            Span::styled(format!("{:.0}%", ratio), theme.value_style()),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[4 + i]);
    }
}

/// Render extended summary statistics
#[allow(dead_code)]
pub fn render_extended_summary(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // Web searches
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Longest session header
        Constraint::Length(1), // Duration
        Constraint::Length(1), // Messages
        Constraint::Min(0),
    ])
    .split(area);

    // Web Searches
    let line = Line::from(vec![
        Span::styled("Web Searches:", theme.label_style()),
        Span::styled(
            format!(" {}", format_number(app.web_search_stats.total_searches)),
            theme.value_style(),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Longest Session header
    let line = Line::from(vec![Span::styled(
        "── Longest Session ──",
        theme.label_style(),
    )]);
    frame.render_widget(Paragraph::new(line), chunks[2]);

    // Longest session details
    if let Some(ref stats) = app.stats {
        if let Some(ref longest) = stats.longest_session {
            let duration = crate::data::format_duration_ms(longest.duration);
            let line = Line::from(vec![
                Span::styled("Duration:    ", theme.label_style()),
                Span::styled(duration, theme.highlight_style()),
            ]);
            frame.render_widget(Paragraph::new(line), chunks[3]);

            let line = Line::from(vec![
                Span::styled("Messages:    ", theme.label_style()),
                Span::styled(format_number(longest.message_count), theme.value_style()),
            ]);
            frame.render_widget(Paragraph::new(line), chunks[4]);
        } else {
            let line = Line::from(vec![Span::styled("No session data", theme.label_style())]);
            frame.render_widget(Paragraph::new(line), chunks[3]);
        }
    }
}

/// Render top projects
#[allow(dead_code)]
pub fn render_top_projects(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut constraints: Vec<Constraint> = vec![Constraint::Length(1)]; // Header
    constraints.extend((0..app.top_projects.len().min(5)).map(|_| Constraint::Length(1)));
    constraints.push(Constraint::Min(0));

    let chunks = Layout::vertical(constraints).split(area);

    // Header
    let line = Line::from(vec![Span::styled(
        "── Top Projects ──",
        theme.label_style(),
    )]);
    frame.render_widget(Paragraph::new(line), chunks[0]);

    // Projects
    for (i, project) in app.top_projects.iter().take(5).enumerate() {
        let line = Line::from(vec![
            Span::styled(
                format!("{:<14}", truncate_str(&project.name, 14)),
                theme.value_style(),
            ),
            Span::styled(
                format!(" {:>5}", format_number(project.message_count)),
                theme.label_style(),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i + 1]);
    }
}

#[allow(dead_code)]
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
