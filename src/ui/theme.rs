use ratatui::style::{Color, Modifier, Style};

#[allow(dead_code)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_focused: Color,
    pub background: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_muted: Color::DarkGray,
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            background: Color::Reset,
        }
    }
}

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn label_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn value_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    #[allow(dead_code)]
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Style for upward trends (positive change)
    pub fn trend_up_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Style for downward trends (negative change)
    pub fn trend_down_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Style for flat/no change trends
    pub fn trend_flat_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    /// Style for safe/OK status indicators
    pub fn safe_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for warning/danger status indicators
    pub fn danger_style(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for sparkline bars
    pub fn sparkline_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    #[allow(dead_code)]
    pub fn bar_style(&self, index: usize) -> Style {
        let colors = [
            self.primary,
            self.accent,
            self.success,
            self.warning,
            self.secondary,
        ];
        Style::default().fg(colors[index % colors.len()])
    }

    pub fn model_color(&self, model_name: &str) -> Color {
        if model_name.contains("opus") {
            self.accent
        } else if model_name.contains("sonnet-4-5") {
            self.primary
        } else if model_name.contains("sonnet") {
            self.secondary
        } else if model_name.contains("haiku") {
            self.success
        } else {
            self.text
        }
    }
}
