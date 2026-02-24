use crate::config::ThemePreset;
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

impl Theme {
    pub fn from_preset(preset: &ThemePreset) -> Self {
        match preset {
            ThemePreset::Default => Self {
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
            },
            ThemePreset::Catppuccin => Self {
                primary: Color::Rgb(137, 180, 250),        // blue
                secondary: Color::Rgb(245, 194, 231),      // pink
                accent: Color::Rgb(203, 166, 247),         // mauve
                success: Color::Rgb(166, 227, 161),        // green
                warning: Color::Rgb(249, 226, 175),        // yellow
                error: Color::Rgb(243, 139, 168),          // red
                text: Color::Rgb(205, 214, 244),           // text
                text_muted: Color::Rgb(166, 173, 200),     // subtext0
                border: Color::Rgb(88, 91, 112),           // surface2
                border_focused: Color::Rgb(137, 180, 250), // blue
                background: Color::Rgb(30, 30, 46),        // base
            },
            ThemePreset::Dracula => Self {
                primary: Color::Rgb(139, 233, 253),        // cyan
                secondary: Color::Rgb(189, 147, 249),      // purple
                accent: Color::Rgb(255, 121, 198),         // pink
                success: Color::Rgb(80, 250, 123),         // green
                warning: Color::Rgb(241, 250, 140),        // yellow
                error: Color::Rgb(255, 85, 85),            // red
                text: Color::Rgb(248, 248, 242),           // foreground
                text_muted: Color::Rgb(98, 114, 164),      // comment
                border: Color::Rgb(68, 71, 90),            // current line
                border_focused: Color::Rgb(139, 233, 253), // cyan
                background: Color::Rgb(40, 42, 54),        // background
            },
            ThemePreset::Nord => Self {
                primary: Color::Rgb(136, 192, 208),        // frost 8
                secondary: Color::Rgb(129, 161, 193),      // frost 9
                accent: Color::Rgb(180, 142, 173),         // aurora 15
                success: Color::Rgb(163, 190, 140),        // aurora 14
                warning: Color::Rgb(235, 203, 139),        // aurora 13
                error: Color::Rgb(191, 97, 106),           // aurora 11
                text: Color::Rgb(216, 222, 233),           // snow storm 4
                text_muted: Color::Rgb(76, 86, 106),       // polar night 3
                border: Color::Rgb(76, 86, 106),           // polar night 3
                border_focused: Color::Rgb(136, 192, 208), // frost 8
                background: Color::Rgb(46, 52, 64),        // polar night 0
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_preset(&ThemePreset::Default)
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
