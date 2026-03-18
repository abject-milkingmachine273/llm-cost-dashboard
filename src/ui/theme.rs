use ratatui::style::{Color, Modifier, Style};

/// Centralised colour and style palette for the dashboard.
///
/// All widgets should source their styles from this struct so that the visual
/// theme can be changed in one place.
pub struct Theme;

impl Theme {
    /// Bold cyan style used for the top title bar.
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// Bold yellow style used for table column headers.
    pub fn header() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    /// Green style indicating a healthy / within-budget state.
    pub fn ok() -> Style {
        Style::default().fg(Color::Green)
    }

    /// Yellow style indicating a warning state (e.g. alert threshold crossed).
    pub fn warn() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// Red style indicating an error or over-budget state.
    pub fn danger() -> Style {
        Style::default().fg(Color::Red)
    }

    /// Standard white foreground for ordinary body text.
    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }

    /// Dark-grey style used for labels and secondary information.
    pub fn dim() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Cyan-on-black style used to highlight a selected table row.
    pub fn highlight() -> Style {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    }

    /// Dark-grey style for widget borders.
    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Choose `ok`, `warn`, or `danger` based on the fraction of budget consumed.
    ///
    /// - `pct < 0.8` returns [`Theme::ok`]
    /// - `0.8 <= pct < 1.0` returns [`Theme::warn`]
    /// - `pct >= 1.0` returns [`Theme::danger`]
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
