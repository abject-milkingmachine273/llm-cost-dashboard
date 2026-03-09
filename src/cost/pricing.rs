//! Per-model token pricing table (USD per 1M tokens).

/// `(model_id, input_usd_per_1m, output_usd_per_1m)`
pub const PRICING: &[(&str, f64, f64)] = &[
    ("claude-opus-4-6",    15.00, 75.00),
    ("claude-sonnet-4-6",   3.00, 15.00),
    ("claude-haiku-4-5",    0.25,  1.25),
    ("gpt-4o",              5.00, 15.00),
    ("gpt-4o-mini",         0.15,  0.60),
    ("gpt-4-turbo",        10.00, 30.00),
    ("o1-preview",         15.00, 60.00),
    ("o3-mini",             1.10,  4.40),
    ("gemini-1.5-pro",      3.50, 10.50),
    ("gemini-1.5-flash",    0.075, 0.30),
];

/// Fallback pricing for unknown models (mid-range estimate).
pub const FALLBACK_PRICING: (f64, f64) = (5.00, 15.00);

/// Look up pricing for a model. Returns `(input_per_1m, output_per_1m)`.
pub fn lookup(model: &str) -> (f64, f64) {
    PRICING
        .iter()
        .find(|(m, _, _)| m.eq_ignore_ascii_case(model))
        .map(|(_, i, o)| (*i, *o))
        .unwrap_or(FALLBACK_PRICING)
}

/// Compute cost in USD from token counts.
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
}
