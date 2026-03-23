//! # FinOps Cost Tagging and Attribution
//!
//! Adds structured tag-based cost attribution so teams can track LLM spend
//! by project, team, cost centre, environment, and any custom dimension.
//!
//! ## Why tagging?
//!
//! A shared LLM deployment serving multiple teams produces a single cost
//! signal.  Without tagging it is impossible to answer:
//!
//! - Which project drove the 40% cost spike last Thursday?
//! - What fraction of the monthly bill belongs to the production environment?
//! - Which team is over their per-sprint LLM budget?
//!
//! Tagging solves this by attaching key-value labels to each [`CostRecord`]
//! as it is ingested and providing a roll-up engine that aggregates cost by
//! any tag dimension.
//!
//! ## Tag sources
//!
//! Tags can come from:
//!
//! 1. **NDJSON log fields** — extra fields in the log line are captured automatically.
//! 2. **Request metadata** — caller-supplied key-value pairs added at call time.
//! 3. **Inference rules** — a [`TagRule`] maps field patterns to tags.
//! 4. **Default tags** — always-present tags like `env=production`.
//!
//! ## Example
//!
//! ```rust
//! use llm_cost_dashboard::tagging::{TagEngine, TagRule, TagMatch, TagSet};
//!
//! let mut engine = TagEngine::new();
//!
//! // Always tag with the environment.
//! engine.add_default_tag("env", "production");
//!
//! // Map model names to cost centres.
//! engine.add_rule(TagRule {
//!     field: "model".to_string(),
//!     pattern: TagMatch::Contains("claude".to_string()),
//!     tag_key: "provider".to_string(),
//!     tag_value: "anthropic".to_string(),
//! });
//! engine.add_rule(TagRule {
//!     field: "model".to_string(),
//!     pattern: TagMatch::Contains("gpt".to_string()),
//!     tag_key: "provider".to_string(),
//!     tag_value: "openai".to_string(),
//! });
//!
//! // Resolve tags for a log record.
//! let mut fields = std::collections::HashMap::new();
//! fields.insert("model".to_string(), "claude-sonnet-4-6".to_string());
//! fields.insert("project".to_string(), "recommendation-engine".to_string());
//!
//! let tags = engine.resolve(&fields);
//! assert_eq!(tags.get("provider"), Some(&"anthropic".to_string()));
//! assert_eq!(tags.get("env"), Some(&"production".to_string()));
//! assert_eq!(tags.get("project"), Some(&"recommendation-engine".to_string()));
//! ```

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TagSet
// ---------------------------------------------------------------------------

/// An immutable snapshot of key-value tags attached to a cost record.
pub type TagSet = HashMap<String, String>;

// ---------------------------------------------------------------------------
// TagMatch
// ---------------------------------------------------------------------------

/// Pattern used by a [`TagRule`] to match a field value.
#[derive(Debug, Clone)]
pub enum TagMatch {
    /// Exact string equality.
    Exact(String),
    /// The field value contains this substring (case-insensitive).
    Contains(String),
    /// The field value starts with this prefix (case-insensitive).
    Prefix(String),
    /// Always matches — use for rules that apply unconditionally.
    Always,
}

impl TagMatch {
    /// Return `true` if `value` satisfies this match pattern.
    pub fn matches(&self, value: &str) -> bool {
        let lower = value.to_lowercase();
        match self {
            TagMatch::Exact(s) => value == s.as_str(),
            TagMatch::Contains(s) => lower.contains(s.to_lowercase().as_str()),
            TagMatch::Prefix(s) => lower.starts_with(s.to_lowercase().as_str()),
            TagMatch::Always => true,
        }
    }
}

// ---------------------------------------------------------------------------
// TagRule
// ---------------------------------------------------------------------------

/// A rule that derives a tag from a log field value.
///
/// When `pattern` matches the value of `field`, the output tag
/// `tag_key = tag_value` is added to the resolved [`TagSet`].
#[derive(Debug, Clone)]
pub struct TagRule {
    /// The log field to examine (e.g. `"model"`, `"provider"`, `"request_id"`).
    pub field: String,
    /// Pattern that the field value must satisfy.
    pub pattern: TagMatch,
    /// Key of the derived tag.
    pub tag_key: String,
    /// Value of the derived tag.
    pub tag_value: String,
}

// ---------------------------------------------------------------------------
// TagEngine
// ---------------------------------------------------------------------------

/// Resolves a full [`TagSet`] from raw log fields.
///
/// Apply in order:
/// 1. Copy all raw `fields` that are *also* in `passthrough_fields` directly
///    into the tag set (opt-in field pass-through).
/// 2. Apply default tags (always present).
/// 3. Evaluate each [`TagRule`] against the fields.
/// 4. Allow caller-supplied override tags to win on conflict.
///
/// See the [module documentation][self] for a full usage example.
#[derive(Debug, Clone, Default)]
pub struct TagEngine {
    default_tags: TagSet,
    rules: Vec<TagRule>,
    /// Fields whose values are passed through directly as tags.
    passthrough_fields: Vec<String>,
}

impl TagEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a default tag that is present on every resolved [`TagSet`].
    pub fn add_default_tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.default_tags.insert(key.into(), value.into());
    }

    /// Add a rule that derives a tag from a matching log field.
    pub fn add_rule(&mut self, rule: TagRule) {
        self.rules.push(rule);
    }

    /// Pass-through a field: if the field is present in the log record,
    /// copy it as a tag with the same key.
    ///
    /// Useful for fields like `project`, `team`, `cost_centre` that are
    /// already correct in the log and need no derivation.
    pub fn add_passthrough(&mut self, field: impl Into<String>) {
        self.passthrough_fields.push(field.into());
    }

    /// Resolve a full [`TagSet`] from a raw field map.
    ///
    /// `overrides` are applied last and win on conflict with default tags and
    /// rule-derived tags.
    pub fn resolve(&self, fields: &HashMap<String, String>) -> TagSet {
        let mut tags = self.default_tags.clone();

        // 1. Passthrough fields.
        for key in &self.passthrough_fields {
            if let Some(value) = fields.get(key) {
                tags.insert(key.clone(), value.clone());
            }
        }

        // 2. Rule-derived tags.
        for rule in &self.rules {
            if let Some(field_value) = fields.get(&rule.field) {
                if rule.pattern.matches(field_value) {
                    tags.insert(rule.tag_key.clone(), rule.tag_value.clone());
                }
            }
        }

        tags
    }

    /// Resolve with caller-supplied override tags that win on conflict.
    pub fn resolve_with_overrides(
        &self,
        fields: &HashMap<String, String>,
        overrides: &TagSet,
    ) -> TagSet {
        let mut tags = self.resolve(fields);
        for (k, v) in overrides {
            tags.insert(k.clone(), v.clone());
        }
        tags
    }
}

// ---------------------------------------------------------------------------
// CostByTag
// ---------------------------------------------------------------------------

/// Aggregates cost records by a single tag dimension.
#[derive(Debug, Default, Clone)]
pub struct CostByTag {
    /// Total cost in USD keyed by tag value.
    pub totals: HashMap<String, f64>,
    /// Request count keyed by tag value.
    pub counts: HashMap<String, u64>,
}

impl CostByTag {
    /// Record one cost entry against a tag value.
    pub fn record(&mut self, tag_value: impl Into<String>, cost_usd: f64) {
        let key = tag_value.into();
        *self.totals.entry(key.clone()).or_insert(0.0) += cost_usd;
        *self.counts.entry(key).or_insert(0) += 1;
    }

    /// Average cost per request for a given tag value.  Returns `None` if
    /// no records exist for that value.
    pub fn avg_cost(&self, tag_value: &str) -> Option<f64> {
        let total = self.totals.get(tag_value)?;
        let count = self.counts.get(tag_value)?;
        if *count == 0 {
            None
        } else {
            Some(total / *count as f64)
        }
    }

    /// Tag value with the highest total cost.
    pub fn top_spender(&self) -> Option<(&str, f64)> {
        self.totals
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Sorted list of (tag_value, total_cost_usd) descending by cost.
    pub fn ranked(&self) -> Vec<(String, f64)> {
        let mut v: Vec<(String, f64)> = self.totals
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v
    }
}

// ---------------------------------------------------------------------------
// TaggedLedger
// ---------------------------------------------------------------------------

/// An append-only ledger of tagged cost entries.
///
/// Each entry stores the cost, timestamp, and full [`TagSet`].  Supports
/// slicing by any tag dimension for reporting.
#[derive(Debug, Default, Clone)]
pub struct TaggedLedger {
    entries: Vec<TaggedEntry>,
}

/// A single entry in the [`TaggedLedger`].
#[derive(Debug, Clone)]
pub struct TaggedEntry {
    /// Cost in USD for this request.
    pub cost_usd: f64,
    /// Unix timestamp in seconds.
    pub timestamp_s: i64,
    /// Tags attached to this entry.
    pub tags: TagSet,
}

impl TaggedLedger {
    /// Create an empty ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one entry.
    pub fn add(&mut self, cost_usd: f64, timestamp_s: i64, tags: TagSet) {
        self.entries.push(TaggedEntry { cost_usd, timestamp_s, tags });
    }

    /// Aggregate all entries by a single tag dimension.
    pub fn by_tag(&self, tag_key: &str) -> CostByTag {
        let mut agg = CostByTag::default();
        for entry in &self.entries {
            let value = entry.tags.get(tag_key).cloned().unwrap_or_else(|| "untagged".into());
            agg.record(value, entry.cost_usd);
        }
        agg
    }

    /// Total cost across all entries.
    pub fn total_cost_usd(&self) -> f64 {
        self.entries.iter().map(|e| e.cost_usd).sum()
    }

    /// Filter entries to those matching `tag_key = tag_value`.
    pub fn filter_by_tag(&self, tag_key: &str, tag_value: &str) -> Vec<&TaggedEntry> {
        self.entries
            .iter()
            .filter(|e| e.tags.get(tag_key).map(|v| v == tag_value).unwrap_or(false))
            .collect()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if the ledger contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> TagEngine {
        let mut e = TagEngine::new();
        e.add_default_tag("env", "test");
        e.add_passthrough("project");
        e.add_rule(TagRule {
            field: "model".to_string(),
            pattern: TagMatch::Contains("claude".to_string()),
            tag_key: "provider".to_string(),
            tag_value: "anthropic".to_string(),
        });
        e
    }

    #[test]
    fn default_tags_always_present() {
        let e = engine();
        let tags = e.resolve(&HashMap::new());
        assert_eq!(tags.get("env").map(|s| s.as_str()), Some("test"));
    }

    #[test]
    fn passthrough_copies_field() {
        let e = engine();
        let mut fields = HashMap::new();
        fields.insert("project".into(), "billing".into());
        let tags = e.resolve(&fields);
        assert_eq!(tags.get("project").map(|s| s.as_str()), Some("billing"));
    }

    #[test]
    fn rule_derives_tag() {
        let e = engine();
        let mut fields = HashMap::new();
        fields.insert("model".into(), "claude-sonnet-4-6".into());
        let tags = e.resolve(&fields);
        assert_eq!(tags.get("provider").map(|s| s.as_str()), Some("anthropic"));
    }

    #[test]
    fn overrides_win_on_conflict() {
        let e = engine();
        let fields = HashMap::new();
        let mut overrides = HashMap::new();
        overrides.insert("env".into(), "production".into());
        let tags = e.resolve_with_overrides(&fields, &overrides);
        assert_eq!(tags.get("env").map(|s| s.as_str()), Some("production"));
    }

    #[test]
    fn cost_by_tag_aggregates_correctly() {
        let mut ledger = TaggedLedger::new();
        let mut t1 = HashMap::new();
        t1.insert("team".into(), "search".into());
        ledger.add(0.10, 0, t1.clone());
        ledger.add(0.20, 1, t1.clone());
        let mut t2 = HashMap::new();
        t2.insert("team".into(), "billing".into());
        ledger.add(0.05, 2, t2);

        let by_team = ledger.by_tag("team");
        assert!((by_team.totals["search"] - 0.30).abs() < 1e-9);
        assert!((by_team.totals["billing"] - 0.05).abs() < 1e-9);
        let top = by_team.top_spender();
        assert_eq!(top.map(|(k, _)| k), Some("search"));
    }

    #[test]
    fn filter_by_tag_returns_matching() {
        let mut ledger = TaggedLedger::new();
        let mut t = HashMap::new();
        t.insert("env".into(), "production".into());
        ledger.add(1.0, 0, t);
        let mut t2 = HashMap::new();
        t2.insert("env".into(), "staging".into());
        ledger.add(2.0, 1, t2);

        let prod = ledger.filter_by_tag("env", "production");
        assert_eq!(prod.len(), 1);
        assert!((prod[0].cost_usd - 1.0).abs() < 1e-9);
    }

    #[test]
    fn total_cost_sums_all_entries() {
        let mut ledger = TaggedLedger::new();
        ledger.add(0.5, 0, HashMap::new());
        ledger.add(1.5, 1, HashMap::new());
        assert!((ledger.total_cost_usd() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn ranked_sorted_descending() {
        let mut agg = CostByTag::default();
        agg.record("a", 0.5);
        agg.record("b", 2.0);
        agg.record("c", 0.1);
        let ranked = agg.ranked();
        assert_eq!(ranked[0].0, "b");
        assert_eq!(ranked[2].0, "c");
    }
}
