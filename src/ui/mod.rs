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
    pub ledger: CostLedger,
    pub log: RequestLog,
    pub budget: BudgetEnvelope,
    pub scroll_offset: usize,
    pub running: bool,
}

impl App {
    pub fn new(budget_usd: f64) -> Self {
        Self {
            ledger: CostLedger::new(),
            log: RequestLog::new(),
            budget: BudgetEnvelope::new("Monthly", budget_usd, 0.8),
            scroll_offset: 0,
            running: true,
        }
    }

    /// Inject a cost record (and update the budget).
    pub fn record(&mut self, record: CostRecord) {
        let cost = record.total_cost_usd;
        let _ = self.ledger.add(record);
        let _ = self.budget.spend(cost);
    }

    /// Ingest a raw JSON line from stdin/log file.
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

    /// Inject synthetic demo records.
    pub fn load_demo_data(&mut self) {
        let demos: &[(&str, &str, u64, u64, u64)] = &[
            ("claude-sonnet-4-6", "anthropic", 847, 312, 45),
            ("gpt-4o-mini",       "openai",    512, 128, 12),
            ("claude-haiku-4-5",  "anthropic", 256, 64,  8),
            ("claude-sonnet-4-6", "anthropic", 1024, 512, 120),
            ("gpt-4o",            "openai",    2048, 1024, 340),
            ("gpt-4o-mini",       "openai",    400, 200, 15),
            ("claude-sonnet-4-6", "anthropic", 600, 300, 55),
            ("o3-mini",           "openai",    800, 400, 200),
            ("gpt-4o-mini",       "openai",    300, 150, 10),
            ("claude-haiku-4-5",  "anthropic", 128, 32,  5),
            ("claude-sonnet-4-6", "anthropic", 512, 256, 40),
            ("gpt-4o",            "openai",    1024, 512, 180),
            ("gemini-1.5-flash",  "google",    700, 350, 30),
            ("claude-sonnet-4-6", "anthropic", 900, 450, 95),
            ("gpt-4o-mini",       "openai",    600, 300, 22),
            ("claude-haiku-4-5",  "anthropic", 200, 100, 7),
            ("o3-mini",           "openai",    1500, 750, 280),
            ("claude-sonnet-4-6", "anthropic", 750, 375, 65),
            ("gpt-4o",            "openai",    512, 256, 110),
            ("gemini-1.5-pro",    "google",    1000, 500, 150),
        ];
        for (model, provider, inp, out, lat) in demos {
            self.record(CostRecord::new(*model, *provider, *inp, *out, *lat));
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn reset(&mut self) {
        self.ledger.clear();
        self.log.clear();
        self.budget.reset();
        self.scroll_offset = 0;
    }
}

/// Run the TUI event loop.
pub fn run(mut app: App) -> Result<(), DashboardError> {
    crossterm::terminal::enable_raw_mode()
        .map_err(|e| DashboardError::Terminal(e.to_string()))?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .map_err(|e| DashboardError::Terminal(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| DashboardError::Terminal(e.to_string()))?;

    let result = event_loop(&mut terminal, &mut app);

    crossterm::terminal::disable_raw_mode()
        .map_err(|e| DashboardError::Terminal(e.to_string()))?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .map_err(|e| DashboardError::Terminal(e.to_string()))?;

    result
}

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
