//! # llm-cost-dashboard
//!
//! Real-time terminal dashboard for LLM token spend.
//!
//! ## Related Projects
//! - [Reddit-Options-Trader-ROT](https://github.com/Mattbusel/Reddit-Options-Trader-ROT-)
//! - [tokio-prompt-orchestrator](https://github.com/Mattbusel/tokio-prompt-orchestrator)
//! - [rot-signals-api](https://github.com/Mattbusel/rot-signals-api)

pub mod budget;
pub mod cost;
pub mod error;
pub mod log;
pub mod trace;
pub mod ui;

pub use budget::BudgetEnvelope;
pub use cost::{CostLedger, CostRecord, ModelStats};
pub use error::DashboardError;
pub use log::{LogEntry, RequestLog};
pub use trace::{SpanStore, TraceSpan};
pub use ui::App;
