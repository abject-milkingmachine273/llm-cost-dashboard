//! Unified error type for the LLM cost dashboard.

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Cost ledger error: {0}")]
    Ledger(String),

    #[error("Budget exceeded: spent ${spent:.4} of ${limit:.4} limit")]
    BudgetExceeded { spent: f64, limit: f64 },

    #[error("Budget alert: {pct:.1}% of ${limit:.4} consumed")]
    BudgetAlert { pct: f64, limit: f64 },

    #[error("Unknown model '{0}' — using fallback pricing")]
    UnknownModel(String),

    #[error("Log parse error: {0}")]
    LogParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),
}
