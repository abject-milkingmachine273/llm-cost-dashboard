//! # HTTP API Server
//!
//! Optional HTTP server started when the `--serve` CLI flag is provided.
//!
//! Endpoints:
//! - `GET /api/summary`      – JSON summary of current costs.
//! - `GET /api/export.json`  – Full ledger as JSON download.
//! - `GET /api/export.csv`   – Full ledger as CSV download.
//!
//! The server shares ledger state via an `Arc<Mutex<CostLedger>>` so that the
//! TUI and API can coexist in separate Tokio tasks.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::{Arc, Mutex};
//! use llm_cost_dashboard::cost::CostLedger;
//! use llm_cost_dashboard::api::serve;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let ledger = Arc::new(Mutex::new(CostLedger::new()));
//! serve(ledger, 8080).await.unwrap();
//! # }
//! ```

use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tracing::info;

use crate::cost::CostLedger;
use crate::error::DashboardError;

/// Shared application state for the HTTP handlers.
#[derive(Clone)]
pub struct ApiState {
    ledger: Arc<Mutex<CostLedger>>,
}

/// JSON body returned by `GET /api/summary`.
#[derive(Serialize)]
pub struct SummaryResponse {
    /// Total spend across all recorded requests in USD.
    pub total_usd: f64,
    /// Projected 30-day spend based on the last hour of activity.
    pub projected_monthly_usd: f64,
    /// Total number of recorded requests.
    pub request_count: usize,
    /// 7-day daily spend trend (oldest → today).
    pub seven_day_trend: [f64; 7],
}

/// Start the Axum HTTP server on `port`, serving the shared `ledger`.
///
/// This function runs forever (until the process exits).  Callers should
/// spawn it as a background Tokio task alongside the TUI event loop:
///
/// ```no_run
/// # use std::sync::{Arc, Mutex};
/// # use llm_cost_dashboard::cost::CostLedger;
/// # use llm_cost_dashboard::api::serve;
/// # #[tokio::main]
/// # async fn main() {
/// let ledger = Arc::new(Mutex::new(CostLedger::new()));
/// let ledger_api = Arc::clone(&ledger);
/// tokio::spawn(async move { serve(ledger_api, 8080).await.unwrap() });
/// # }
/// ```
///
/// # Errors
///
/// Returns [`DashboardError::Terminal`] if the TCP listener cannot be bound.
pub async fn serve(
    ledger: Arc<Mutex<CostLedger>>,
    port: u16,
) -> Result<(), DashboardError> {
    let state = ApiState { ledger };
    let app = Router::new()
        .route("/api/summary", get(handle_summary))
        .route("/api/export.json", get(handle_export_json))
        .route("/api/export.csv", get(handle_export_csv))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    info!(addr = %addr, "HTTP API server starting");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| DashboardError::Terminal(format!("bind {addr}: {e}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| DashboardError::Terminal(e.to_string()))
}

async fn handle_summary(State(state): State<ApiState>) -> impl IntoResponse {
    let ledger = state.ledger.lock().unwrap_or_else(|e| e.into_inner());
    Json(SummaryResponse {
        total_usd: ledger.total_usd(),
        projected_monthly_usd: ledger.projected_monthly_usd(1),
        request_count: ledger.len(),
        seven_day_trend: ledger.seven_day_trend(),
    })
}

async fn handle_export_json(State(state): State<ApiState>) -> Response {
    let ledger = state.ledger.lock().unwrap_or_else(|e| e.into_inner());
    match ledger.to_json() {
        Ok(json) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/json"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"llm-costs.json\"",
                ),
            ],
            json,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn handle_export_csv(State(state): State<ApiState>) -> Response {
    let ledger = state.ledger.lock().unwrap_or_else(|e| e.into_inner());
    match ledger.to_csv() {
        Ok(csv) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/csv"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"llm-costs.csv\"",
                ),
            ],
            csv,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::CostRecord;

    fn make_state() -> ApiState {
        let mut ledger = CostLedger::new();
        ledger
            .add(CostRecord::new("gpt-4o-mini", "openai", 512, 256, 20))
            .unwrap();
        ApiState {
            ledger: Arc::new(Mutex::new(ledger)),
        }
    }

    #[test]
    fn test_state_ledger_accessible() {
        let state = make_state();
        let l = state.ledger.lock().unwrap();
        assert_eq!(l.len(), 1);
    }

    #[test]
    fn test_summary_response_fields() {
        let state = make_state();
        let l = state.ledger.lock().unwrap();
        let resp = SummaryResponse {
            total_usd: l.total_usd(),
            projected_monthly_usd: l.projected_monthly_usd(1),
            request_count: l.len(),
            seven_day_trend: l.seven_day_trend(),
        };
        assert_eq!(resp.request_count, 1);
        assert!(resp.total_usd > 0.0);
        // Today should have a non-zero value in the trend
        let today_val = resp.seven_day_trend[6];
        assert!(today_val > 0.0);
    }

    #[test]
    fn test_csv_export_contains_header() {
        let state = make_state();
        let l = state.ledger.lock().unwrap();
        let csv = l.to_csv().unwrap();
        assert!(csv.contains("model"));
        assert!(csv.contains("gpt-4o-mini"));
    }

    #[test]
    fn test_json_export_contains_data() {
        let state = make_state();
        let l = state.ledger.lock().unwrap();
        let json = l.to_json().unwrap();
        assert!(json.contains("gpt-4o-mini"));
    }
}
