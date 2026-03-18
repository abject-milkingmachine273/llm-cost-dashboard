//! Integration tests for llm-cost-dashboard.
//!
//! These tests verify cross-module behaviour: demo mode producing real
//! CostRecord entries and the pricing table covering all major models.

use llm_cost_dashboard::{
    cost::pricing::{lookup, FALLBACK_PRICING},
    ui::App,
};

/// Instantiate the app in demo mode and verify it generates at least one
/// cost entry without panicking.
#[test]
fn test_demo_mode_produces_output() {
    let mut app = App::new(50.0);
    app.load_demo_data();

    // The ledger should contain at least one record.
    assert!(
        app.ledger.len() > 0,
        "demo mode should populate the ledger with at least one record"
    );

    // Total spend must be strictly positive.
    assert!(
        app.ledger.total_usd() > 0.0,
        "total spend after demo load must be > $0.00"
    );

    // The log should also have been populated (one log entry per ingest).
    // Demo data uses `App::record` directly (bypasses log ingestion), so
    // we just confirm the ledger path is clean.
    assert_eq!(
        app.ledger.len(),
        20,
        "load_demo_data should inject exactly 20 demo records"
    );
}

/// Verify that every major model family has an explicit entry in the pricing
/// table (i.e. lookup does NOT fall back to FALLBACK_PRICING for these).
#[test]
fn test_pricing_covers_all_major_models() {
    let major_models = [
        "gpt-4o",
        "gpt-4o-mini",
        "gpt-4-turbo",
        "claude-sonnet-4-6",
        "claude-opus-4-6",
        "claude-haiku-4-5",
        "o1-preview",
        "o3-mini",
        "gemini-1.5-pro",
        "gemini-1.5-flash",
    ];

    for model in &major_models {
        let (i, o) = lookup(model);
        assert!(
            (i, o) != FALLBACK_PRICING || {
                // FALLBACK_PRICING coincidentally matches a real model only if
                // the model's rates happen to equal (5.00, 15.00).  Guard that
                // case by also checking the model is actually present in the
                // PRICING table.
                use llm_cost_dashboard::cost::pricing::PRICING;
                PRICING
                    .iter()
                    .any(|(m, _, _)| m.eq_ignore_ascii_case(model))
            },
            "model '{model}' uses fallback pricing — add it to PRICING"
        );
        assert!(i > 0.0, "model '{model}' input rate must be > 0");
        assert!(o > 0.0, "model '{model}' output rate must be > 0");
    }
}

/// Demo mode should not exceed the budget with the default $50 limit.
#[test]
fn test_demo_mode_does_not_panic_with_low_budget() {
    // Budget will be exceeded but the app must not panic — errors are
    // silently absorbed via `let _ = self.budget.spend(cost)`.
    let mut app = App::new(0.0001);
    app.load_demo_data();
    // If we get here, no panic occurred.
    assert!(app.ledger.len() > 0);
}

/// `ingest_line` with valid JSON must add exactly one record to both the
/// log and the ledger and must not panic.
#[test]
fn test_ingest_line_integration() {
    let mut app = App::new(100.0);
    let line =
        r#"{"model":"gpt-4o-mini","input_tokens":1000,"output_tokens":500,"latency_ms":20}"#;
    app.ingest_line(line).expect("valid JSON line should be ingested");
    assert_eq!(app.log.len(), 1);
    assert_eq!(app.ledger.len(), 1);
    assert!(app.ledger.total_usd() > 0.0);
}

/// Ingesting a malformed JSON line must return an error but must not corrupt
/// the ledger or the log.
#[test]
fn test_ingest_malformed_line_does_not_corrupt_state() {
    let mut app = App::new(100.0);
    // First add a valid record.
    let good = r#"{"model":"gpt-4o","input_tokens":100,"output_tokens":50,"latency_ms":10}"#;
    app.ingest_line(good).unwrap();

    // Now try a bad record.
    let bad = "this is not json";
    assert!(app.ingest_line(bad).is_err());

    // The valid record must still be there.
    assert_eq!(app.ledger.len(), 1);
    assert_eq!(app.log.len(), 1);
}

/// Resetting the app after loading demo data must return it to a clean state.
#[test]
fn test_reset_after_demo_data() {
    let mut app = App::new(50.0);
    app.load_demo_data();
    assert!(app.ledger.len() > 0);
    app.reset();
    assert_eq!(app.ledger.len(), 0);
    assert_eq!(app.log.len(), 0);
    assert_eq!(app.ledger.total_usd(), 0.0);
    assert_eq!(app.budget.spent_usd, 0.0);
    assert_eq!(app.scroll_offset, 0);
}
