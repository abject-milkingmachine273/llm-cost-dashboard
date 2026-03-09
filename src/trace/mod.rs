//! # Trace Spans
//!
//! Distributed tracing for LLM requests. Each span records a request_id,
//! model, latency, token counts, and computed cost.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cost::pricing::compute_cost;
use crate::error::DashboardError;

/// A single traced LLM request span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub span_id: Uuid,
    pub request_id: String,
    pub model: String,
    pub provider: String,
    pub started_at: DateTime<Utc>,
    pub latency_ms: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub success: bool,
    pub tags: Vec<String>,
}

impl TraceSpan {
    pub fn new(
        request_id: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
        input_tokens: u64,
        output_tokens: u64,
        latency_ms: u64,
    ) -> Self {
        let model = model.into();
        let cost = compute_cost(&model, input_tokens, output_tokens);
        Self {
            span_id: Uuid::new_v4(),
            request_id: request_id.into(),
            model,
            provider: provider.into(),
            started_at: Utc::now(),
            latency_ms,
            input_tokens,
            output_tokens,
            cost_usd: cost,
            success: true,
            tags: Vec::new(),
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn failed(mut self, reason: impl Into<String>) -> Self {
        self.success = false;
        self.tags.push(format!("error:{}", reason.into()));
        self
    }

    pub fn to_json(&self) -> Result<String, DashboardError> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// In-memory span store.
#[derive(Debug, Default)]
pub struct SpanStore {
    spans: Vec<TraceSpan>,
}

impl SpanStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, span: TraceSpan) {
        self.spans.push(span);
    }

    pub fn by_request(&self, request_id: &str) -> Option<&TraceSpan> {
        self.spans.iter().find(|s| s.request_id == request_id)
    }

    pub fn all(&self) -> &[TraceSpan] {
        &self.spans
    }

    pub fn total_cost(&self) -> f64 {
        self.spans.iter().map(|s| s.cost_usd).sum()
    }

    pub fn failure_count(&self) -> usize {
        self.spans.iter().filter(|s| !s.success).count()
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_records_latency() {
        let s = TraceSpan::new("req1", "gpt-4o", "openai", 100, 50, 42);
        assert_eq!(s.latency_ms, 42);
    }

    #[test]
    fn test_span_cost_computed_from_tokens() {
        let s = TraceSpan::new("req1", "claude-sonnet-4-6", "anthropic", 1_000_000, 0, 100);
        assert!((s.cost_usd - 3.00).abs() < 1e-9);
    }

    #[test]
    fn test_span_unknown_model_uses_fallback() {
        let s = TraceSpan::new("req1", "unknown-model", "provider", 1_000_000, 0, 100);
        assert!(s.cost_usd > 0.0);
    }

    #[test]
    fn test_span_to_json_roundtrip() {
        let s = TraceSpan::new("req1", "gpt-4o", "openai", 100, 50, 42);
        let json = s.to_json().unwrap();
        let deserialized: TraceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, "req1");
        assert_eq!(deserialized.latency_ms, 42);
    }

    #[test]
    fn test_span_with_tag() {
        let s = TraceSpan::new("req1", "gpt-4o", "openai", 100, 50, 42).with_tag("production");
        assert!(s.tags.contains(&"production".to_string()));
    }

    #[test]
    fn test_span_failed_marks_success_false() {
        let s = TraceSpan::new("req1", "gpt-4o", "openai", 100, 50, 42).failed("timeout");
        assert!(!s.success);
        assert!(s.tags.iter().any(|t| t.starts_with("error:")));
    }

    #[test]
    fn test_store_record_and_retrieve() {
        let mut store = SpanStore::new();
        store.record(TraceSpan::new("req-abc", "gpt-4o", "openai", 100, 50, 42));
        assert!(store.by_request("req-abc").is_some());
        assert!(store.by_request("req-xyz").is_none());
    }

    #[test]
    fn test_store_total_cost() {
        let mut store = SpanStore::new();
        store.record(TraceSpan::new(
            "r1",
            "claude-sonnet-4-6",
            "anthropic",
            1_000_000,
            0,
            10,
        ));
        store.record(TraceSpan::new(
            "r2",
            "claude-sonnet-4-6",
            "anthropic",
            1_000_000,
            0,
            10,
        ));
        assert!((store.total_cost() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_store_failure_count() {
        let mut store = SpanStore::new();
        store.record(TraceSpan::new("r1", "gpt-4o", "openai", 100, 50, 10).failed("err"));
        store.record(TraceSpan::new("r2", "gpt-4o", "openai", 100, 50, 10));
        assert_eq!(store.failure_count(), 1);
    }
}
