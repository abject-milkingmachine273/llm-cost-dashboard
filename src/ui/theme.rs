use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn title() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }
    pub fn header() -> Style {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    }
    pub fn ok() -> Style {
        Style::default().fg(Color::Green)
    }
    pub fn warn() -> Style {
        Style::default().fg(Color::Yellow)
    }
    pub fn danger() -> Style {
        Style::default().fg(Color::Red)
    }
    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }
    pub fn dim() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn highlight() -> Style {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    }
    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Pick ok/warn/danger style based on fraction consumed.
    pub fn budget_style(pct: f64) -> Style {
        if pct >= 1.0 {
            Self::danger()
        } else if pct >= 0.8 {
            Self::warn()
        } else {
            Self::ok()
        }
    }
}
