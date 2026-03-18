//! # TUI Application
//!
//! Entry point for the ratatui dashboard. [`App`] owns all application state;
//! [`run`] drives the terminal event loop.

pub mod dashboard;
pub mod theme;
pub mod widgets;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    budget::BudgetEnvelope,
    cost::{CostLedger, CostRecord},
    error::DashboardError,
    log::RequestLog,
};

/// Application state — everything the TUI needs.
pub struct App {
    /// The cost ledger tracking every completed request.
    pub ledger: CostLedger,
    /// Raw request log for display and filtering.
    pub log: RequestLog,
    /// Active budget envelope for spend enforcement.
    pub budget: BudgetEnvelope,
    /// Current scroll position in the requests table (rows from top).
    pub scroll_offset: usize,
    /// Whether the event loop should continue running.
    pub running: bool,
}

impl App {
    /// Create a new application with the given monthly budget limit in USD.
    ///
    /// The budget alert threshold is set to 80 % of the limit.
    pub fn new(budget_usd: f64) -> Self {
        Self {
            ledger: CostLedger::new(),
            log: RequestLog::new(),
            budget: BudgetEnvelope::new("Monthly", budget_usd, 0.8),
            scroll_offset: 0,
            running: true,
        }
    }

    /// Inject a [`CostRecord`] into the ledger and update the budget.
    ///
    /// Budget-exceeded errors are silently absorbed so the TUI does not crash;
    /// the budget widget will reflect the over-limit state visually.
    pub fn record(&mut self, record: CostRecord) {
        let cost = record.total_cost_usd;
        let _ = self.ledger.add(record);
        let _ = self.budget.spend(cost);
    }

    /// Parse a raw newline-delimited JSON line and ingest it into both the
    /// request log and the cost ledger.
    ///
    /// # Errors
    ///
    /// Returns [`DashboardError::LogParseError`] when the line cannot be
    /// parsed.  The ledger and log are not modified on error.
    pub fn ingest_line(&mut self, line: &str) -> Result<(), DashboardError> {
        self.log.ingest_line(line)?;
        if let Some(last) = self.log.all().last() {
            let rec = CostRecord::new(
                &last.model,
                &last.provider,
                last.input_tokens,
                last.output_tokens,
                last.latency_ms,
            );
            self.record(rec);
        }
        Ok(())
    }

    /// Populate the app with 20 synthetic demo records covering multiple models
    /// and providers.
    pub fn load_demo_data(&mut self) {
        let demos: &[(&str, &str, u64, u64, u64)] = &[
            ("claude-sonnet-4-6", "anthropic", 847, 312, 45),
            ("gpt-4o-mini", "openai", 512, 128, 12),
            ("claude-haiku-4-5", "anthropic", 256, 64, 8),
            ("claude-sonnet-4-6", "anthropic", 1024, 512, 120),
            ("gpt-4o", "openai", 2048, 1024, 340),
            ("gpt-4o-mini", "openai", 400, 200, 15),
            ("claude-sonnet-4-6", "anthropic", 600, 300, 55),
            ("o3-mini", "openai", 800, 400, 200),
            ("gpt-4o-mini", "openai", 300, 150, 10),
            ("claude-haiku-4-5", "anthropic", 128, 32, 5),
            ("claude-sonnet-4-6", "anthropic", 512, 256, 40),
            ("gpt-4o", "openai", 1024, 512, 180),
            ("gemini-1.5-flash", "google", 700, 350, 30),
            ("claude-sonnet-4-6", "anthropic", 900, 450, 95),
            ("gpt-4o-mini", "openai", 600, 300, 22),
            ("claude-haiku-4-5", "anthropic", 200, 100, 7),
            ("o3-mini", "openai", 1500, 750, 280),
            ("claude-sonnet-4-6", "anthropic", 750, 375, 65),
            ("gpt-4o", "openai", 512, 256, 110),
            ("gemini-1.5-pro", "google", 1000, 500, 150),
        ];
        for (model, provider, inp, out, lat) in demos {
            self.record(CostRecord::new(*model, *provider, *inp, *out, *lat));
        }
    }

    /// Scroll the requests table down by one row.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll the requests table up by one row.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Clear all data and reset scroll position, returning to a blank state.
    pub fn reset(&mut self) {
        self.ledger.clear();
        self.log.clear();
        self.budget.reset();
        self.scroll_offset = 0;
    }
}

/// Run the full-screen TUI event loop.
///
/// Enables raw mode, enters the alternate screen buffer, renders on every
/// tick, and restores the terminal on exit or error.
///
/// # Errors
///
/// Returns [`DashboardError::Terminal`] if any crossterm or ratatui operation
/// fails, or propagates errors from the event loop.
pub fn run(mut app: App) -> Result<(), DashboardError> {
    crossterm::terminal::enable_raw_mode().map_err(|e| DashboardError::Terminal(e.to_string()))?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .map_err(|e| DashboardError::Terminal(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| DashboardError::Terminal(e.to_string()))?;

    let result = event_loop(&mut terminal, &mut app);

    crossterm::terminal::disable_raw_mode().map_err(|e| DashboardError::Terminal(e.to_string()))?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .map_err(|e| DashboardError::Terminal(e.to_string()))?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new_empty() {
        let app = App::new(10.0);
        assert!(app.ledger.is_empty());
        assert!(app.log.is_empty());
        assert_eq!(app.budget.limit_usd, 10.0);
        assert!(app.running);
    }

    #[test]
    fn test_app_record_updates_ledger_and_budget() {
        let mut app = App::new(100.0);
        app.record(CostRecord::new("gpt-4o-mini", "openai", 1_000_000, 0, 10));
        assert_eq!(app.ledger.len(), 1);
        assert!(app.budget.spent_usd > 0.0);
    }

    #[test]
    fn test_app_ingest_line_valid() {
        let mut app = App::new(100.0);
        let line =
            r#"{"model":"gpt-4o-mini","input_tokens":100,"output_tokens":50,"latency_ms":10}"#;
        app.ingest_line(line).unwrap();
        assert_eq!(app.ledger.len(), 1);
        assert_eq!(app.log.len(), 1);
    }

    #[test]
    fn test_app_ingest_line_invalid_does_not_panic() {
        let mut app = App::new(100.0);
        let result = app.ingest_line("not json at all");
        assert!(result.is_err());
        assert!(app.ledger.is_empty());
        assert!(app.log.is_empty());
    }

    #[test]
    fn test_app_ingest_line_malformed_json_is_log_parse_error() {
        let mut app = App::new(100.0);
        let err = app.ingest_line("{bad}").unwrap_err();
        assert!(matches!(err, DashboardError::LogParseError(_)));
    }

    #[test]
    fn test_app_load_demo_data_populates_ledger() {
        let mut app = App::new(100.0);
        app.load_demo_data();
        assert!(app.ledger.len() > 0);
    }

    #[test]
    fn test_app_reset_clears_all_state() {
        let mut app = App::new(100.0);
        app.load_demo_data();
        app.scroll_offset = 5;
        app.reset();
        assert!(app.ledger.is_empty());
        assert!(app.log.is_empty());
        assert_eq!(app.budget.spent_usd, 0.0);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_app_scroll_down_increments() {
        let mut app = App::new(10.0);
        app.scroll_down();
        assert_eq!(app.scroll_offset, 1);
    }

    #[test]
    fn test_app_scroll_up_at_zero_stays_zero() {
        let mut app = App::new(10.0);
        app.scroll_up();
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_app_scroll_down_up_roundtrip() {
        let mut app = App::new(10.0);
        app.scroll_down();
        app.scroll_down();
        app.scroll_up();
        assert_eq!(app.scroll_offset, 1);
    }
}

/// Inner event loop: draw → poll → handle key, repeated until `app.running` is false.
fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<(), DashboardError> {
    while app.running {
        terminal
            .draw(|frame| dashboard::render(frame, &app.ledger, &app.budget, app.scroll_offset))
            .map_err(|e| DashboardError::Terminal(e.to_string()))?;

        if event::poll(Duration::from_millis(250))
            .map_err(|e| DashboardError::Terminal(e.to_string()))?
        {
            match event::read().map_err(|e| DashboardError::Terminal(e.to_string()))? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.running = false,
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('d') => app.load_demo_data(),
                    KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}
