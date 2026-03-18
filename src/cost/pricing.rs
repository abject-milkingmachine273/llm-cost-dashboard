//! Per-model token pricing table (USD per 1 million tokens).
//!
//! All prices are stored as `f64` values representing US dollars per 1,000,000
//! tokens.  Use [`compute_cost`] for convenience or [`lookup`] when you need
//! to inspect the raw rates.

/// Statically known per-model pricing entries.
///
/// Each tuple is `(model_id, input_usd_per_1m, output_usd_per_1m)`.
/// Model IDs are matched case-insensitively by [`lookup`].
///
/// Last updated: 2026-03-18
pub const PRICING: &[(&str, f64, f64)] = &[
    // Anthropic — Claude 4 family
    ("claude-opus-4-6", 15.00, 75.00),
    ("claude-sonnet-4-6", 3.00, 15.00),
    ("claude-haiku-4-5", 0.25, 1.25),
    // OpenAI — GPT-4o family
    ("gpt-4o", 5.00, 15.00),
    ("gpt-4o-mini", 0.15, 0.60),
    ("gpt-4-turbo", 10.00, 30.00),
    // OpenAI — o-series reasoning models
    ("o1", 15.00, 60.00),
    ("o1-mini", 1.10, 4.40),
    ("o3", 10.00, 40.00),
    ("o3-mini", 1.10, 4.40),
    ("o4-mini", 1.10, 4.40),
    // Google — Gemini family
    ("gemini-2.0-flash", 0.10, 0.40),
    ("gemini-2.0-flash-lite", 0.075, 0.30),
    ("gemini-1.5-pro", 3.50, 10.50),
    ("gemini-1.5-flash", 0.075, 0.30),
];

/// Fallback pricing used when the model is not found in [`PRICING`].
///
/// This is a mid-range estimate to avoid wildly incorrect costs for unknown
/// models.  The [`crate::error::DashboardError::UnknownModel`] variant is
/// available for callers that wish to surface the absence explicitly.
pub const FALLBACK_PRICING: (f64, f64) = (5.00, 15.00);

/// Look up pricing for `model`.
///
/// The lookup is case-insensitive.  If the model is not found, returns
/// [`FALLBACK_PRICING`].
///
/// Returns `(input_usd_per_1m_tokens, output_usd_per_1m_tokens)`.
pub fn lookup(model: &str) -> (f64, f64) {
    PRICING
        .iter()
        .find(|(m, _, _)| m.eq_ignore_ascii_case(model))
        .map(|(_, i, o)| (*i, *o))
        .unwrap_or(FALLBACK_PRICING)
}

/// Compute the total cost in USD for the given token counts.
///
/// Uses [`lookup`] internally; unknown models fall back to [`FALLBACK_PRICING`].
///
/// # Examples
///
/// ```
/// use llm_cost_dashboard::cost::pricing::compute_cost;
///
/// let cost = compute_cost("claude-sonnet-4-6", 1_000_000, 0);
/// assert!((cost - 3.00).abs() < 1e-9);
/// ```
pub fn compute_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
    let (input_rate, output_rate) = lookup(model);
    (input_tokens as f64 * input_rate + output_tokens as f64 * output_rate) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_model() {
        let (i, o) = lookup("claude-sonnet-4-6");
        assert!((i - 3.00).abs() < f64::EPSILON);
        assert!((o - 15.00).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let (i1, _) = lookup("GPT-4O");
        let (i2, _) = lookup("gpt-4o");
        assert!((i1 - i2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lookup_unknown_uses_fallback() {
        let (i, o) = lookup("unknown-model-xyz");
        assert_eq!((i, o), FALLBACK_PRICING);
    }

    #[test]
    fn test_compute_cost_zero_tokens() {
        assert_eq!(compute_cost("claude-sonnet-4-6", 0, 0), 0.0);
    }

    #[test]
    fn test_compute_cost_one_million_input() {
        let cost = compute_cost("claude-sonnet-4-6", 1_000_000, 0);
        assert!((cost - 3.00).abs() < 1e-9);
    }

    #[test]
    fn test_compute_cost_one_million_output() {
        let cost = compute_cost("claude-sonnet-4-6", 0, 1_000_000);
        assert!((cost - 15.00).abs() < 1e-9);
    }

    #[test]
    fn test_all_models_have_positive_rates() {
        for (model, i, o) in PRICING {
            assert!(*i > 0.0, "model {model} has zero input rate");
            assert!(*o > 0.0, "model {model} has zero output rate");
        }
    }

    // --- per-model exact pricing tests ---

    #[test]
    fn test_claude_opus_pricing() {
        let (i, o) = lookup("claude-opus-4-6");
        assert!((i - 15.00).abs() < 1e-9);
        assert!((o - 75.00).abs() < 1e-9);
    }

    #[test]
    fn test_claude_haiku_pricing() {
        let (i, o) = lookup("claude-haiku-4-5");
        assert!((i - 0.25).abs() < 1e-9);
        assert!((o - 1.25).abs() < 1e-9);
    }

    #[test]
    fn test_gpt4o_pricing() {
        let (i, o) = lookup("gpt-4o");
        assert!((i - 5.00).abs() < 1e-9);
        assert!((o - 15.00).abs() < 1e-9);
    }

    #[test]
    fn test_gpt4o_mini_pricing() {
        let (i, o) = lookup("gpt-4o-mini");
        assert!((i - 0.15).abs() < 1e-9);
        assert!((o - 0.60).abs() < 1e-9);
    }

    #[test]
    fn test_gpt4_turbo_pricing() {
        let (i, o) = lookup("gpt-4-turbo");
        assert!((i - 10.00).abs() < 1e-9);
        assert!((o - 30.00).abs() < 1e-9);
    }

    #[test]
    fn test_o1_pricing() {
        let (i, o) = lookup("o1");
        assert!((i - 15.00).abs() < 1e-9);
        assert!((o - 60.00).abs() < 1e-9);
    }

    #[test]
    fn test_o3_mini_pricing() {
        let (i, o) = lookup("o3-mini");
        assert!((i - 1.10).abs() < 1e-9);
        assert!((o - 4.40).abs() < 1e-9);
    }

    #[test]
    fn test_gemini_15_pro_pricing() {
        let (i, o) = lookup("gemini-1.5-pro");
        assert!((i - 3.50).abs() < 1e-9);
        assert!((o - 10.50).abs() < 1e-9);
    }

    #[test]
    fn test_gemini_15_flash_pricing() {
        let (i, o) = lookup("gemini-1.5-flash");
        assert!((i - 0.075).abs() < 1e-9);
        assert!((o - 0.30).abs() < 1e-9);
    }

    #[test]
    fn test_gemini_20_flash_pricing() {
        let (i, o) = lookup("gemini-2.0-flash");
        assert!((i - 0.10).abs() < 1e-9);
        assert!((o - 0.40).abs() < 1e-9);
    }

    // --- edge cases ---

    #[test]
    fn test_compute_cost_max_u64_does_not_panic() {
        // u64::MAX tokens should produce a very large but finite cost, not panic.
        let cost = compute_cost("gpt-4o-mini", u64::MAX, 0);
        assert!(cost.is_finite() || cost.is_infinite()); // either is acceptable, just no panic
    }

    #[test]
    fn test_compute_cost_fractional_result() {
        // 1 token at $0.15/1M = $0.00000015
        let cost = compute_cost("gpt-4o-mini", 1, 0);
        assert!((cost - 0.15 / 1_000_000.0).abs() < 1e-15);
    }

    #[test]
    fn test_all_pricing_table_entries_lookable() {
        for (model, expected_i, expected_o) in PRICING {
            let (i, o) = lookup(model);
            assert!((i - expected_i).abs() < 1e-9, "input mismatch for {model}");
            assert!((o - expected_o).abs() < 1e-9, "output mismatch for {model}");
        }
    }

    // ── Property tests (proptest) ─────────────────────────────────────────────

    proptest::proptest! {
        /// Cost is always non-negative for any non-negative token counts.
        #[test]
        fn prop_cost_non_negative(
            input in 0u64..10_000_000u64,
            output in 0u64..10_000_000u64,
            idx in 0usize..PRICING.len(),
        ) {
            let cost = compute_cost(PRICING[idx].0, input, output);
            proptest::prop_assert!(cost >= 0.0, "cost was negative: {cost}");
        }

        /// Cost scales linearly with input token count: doubling tokens doubles cost.
        #[test]
        fn prop_cost_linear_with_input(
            tokens in 1u64..1_000_000u64,
            idx in 0usize..PRICING.len(),
        ) {
            let m = PRICING[idx].0;
            let c1 = compute_cost(m, tokens, 0);
            let c2 = compute_cost(m, tokens * 2, 0);
            proptest::prop_assert!((c2 / c1 - 2.0).abs() < 1e-9);
        }

        /// Cost scales linearly with output token count.
        #[test]
        fn prop_cost_linear_with_output(
            tokens in 1u64..1_000_000u64,
            idx in 0usize..PRICING.len(),
        ) {
            let m = PRICING[idx].0;
            let c1 = compute_cost(m, 0, tokens);
            let c2 = compute_cost(m, 0, tokens * 2);
            proptest::prop_assert!((c2 / c1 - 2.0).abs() < 1e-9);
        }

        /// Per-model rates are consistent: looking up the same model twice
        /// gives identical rates.
        #[test]
        fn prop_lookup_deterministic(idx in 0usize..PRICING.len()) {
            let m = PRICING[idx].0;
            let (i1, o1) = lookup(m);
            let (i2, o2) = lookup(m);
            proptest::prop_assert_eq!(i1, i2);
            proptest::prop_assert_eq!(o1, o2);
        }
    }
}
