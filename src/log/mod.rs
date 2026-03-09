//! # Request Log
//!
//! Ordered append log of raw LLM requests. Supports filter-by-model,
//! JSON serialization, and ingestion from newline-delimited JSON files.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DashboardError;

/// Raw log entry — what arrives from the LLM client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

impl LogEntry {
    pub fn new(model: impl Into<String>, provider: impl Into<String>, input: u64, output: u64, latency_ms: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            model: model.into(),
            provider: provider.into(),
            input_tokens: input,
            output_tokens: output,
            latency_ms,
            success: true,
            error: None,
        }
    }
}

/// Incoming JSON record format (what gets tailed from a log file).
#[derive(Debug, Deserialize)]
pub struct IncomingRecord {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub latency_ms: u64,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl From<IncomingRecord> for LogEntry {
    fn from(r: IncomingRecord) -> Self {
        let success = r.error.is_none();
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            provider: r.provider.unwrap_or_else(|| "unknown".into()),
            model: r.model,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            latency_ms: r.latency_ms,
            success,
            error: r.error,
        }
    }
}

/// Append-only request log.
#[derive(Debug, Default)]
pub struct RequestLog {
    entries: Vec<LogEntry>,
}

impl RequestLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    /// Parse and append a newline-delimited JSON line.
    pub fn ingest_line(&mut self, line: &str) -> Result<(), DashboardError> {
        let record: IncomingRecord = serde_json::from_str(line.trim())
            .map_err(|e| DashboardError::LogParse(e.to_string()))?;
        self.append(record.into());
        Ok(())
    }

    pub fn filter_by_model<'a>(&'a self, model: &'a str) -> impl Iterator<Item = &'a LogEntry> {
        self.entries.iter().filter(move |e| e.model.eq_ignore_ascii_case(model))
    }

    pub fn all(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn to_json(&self) -> Result<String, DashboardError> {
        serde_json::to_string_pretty(&self.entries).map_err(Into::into)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_increases_len() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("gpt-4o", "openai", 100, 50, 20));
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_all_returns_in_order() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("a", "p", 1, 1, 1));
        log.append(LogEntry::new("b", "p", 2, 2, 2));
        let all = log.all();
        assert_eq!(all[0].model, "a");
        assert_eq!(all[1].model, "b");
    }

    #[test]
    fn test_filter_by_model_returns_matching() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("gpt-4o", "openai", 100, 50, 20));
        log.append(LogEntry::new("claude-sonnet-4-6", "anthropic", 100, 50, 20));
        let results: Vec<_> = log.filter_by_model("gpt-4o").collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].model, "gpt-4o");
    }

    #[test]
    fn test_filter_by_model_case_insensitive() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("GPT-4O", "openai", 100, 50, 20));
        let results: Vec<_> = log.filter_by_model("gpt-4o").collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filter_no_match_returns_empty() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("gpt-4o", "openai", 100, 50, 20));
        let results: Vec<_> = log.filter_by_model("claude-sonnet-4-6").collect();
        assert!(results.is_empty());
    }

    #[test]
    fn test_ingest_valid_json_line() {
        let mut log = RequestLog::new();
        let line = r#"{"model":"gpt-4o-mini","input_tokens":512,"output_tokens":256,"latency_ms":34}"#;
        log.ingest_line(line).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log.all()[0].model, "gpt-4o-mini");
        assert_eq!(log.all()[0].input_tokens, 512);
    }

    #[test]
    fn test_ingest_invalid_json_returns_error() {
        let mut log = RequestLog::new();
        assert!(log.ingest_line("not json").is_err());
    }

    #[test]
    fn test_ingest_missing_required_field_returns_error() {
        let mut log = RequestLog::new();
        let line = r#"{"model":"gpt-4o-mini","input_tokens":512}"#; // missing output_tokens
        assert!(log.ingest_line(line).is_err());
    }

    #[test]
    fn test_to_json_roundtrip() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("gpt-4o", "openai", 100, 50, 20));
        let json = log.to_json().unwrap();
        assert!(json.contains("gpt-4o"));
    }

    #[test]
    fn test_clear_empties_log() {
        let mut log = RequestLog::new();
        log.append(LogEntry::new("gpt-4o", "openai", 100, 50, 20));
        log.clear();
        assert!(log.is_empty());
    }
}
