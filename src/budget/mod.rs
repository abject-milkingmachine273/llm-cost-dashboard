//! # Budget Envelope
//!
//! Hard budget enforcement with atomic tracking, alert thresholds,
//! and remaining-balance queries.

use crate::error::DashboardError;

/// A budget envelope with a hard limit and an alert threshold.
#[derive(Debug, Clone)]
pub struct BudgetEnvelope {
    /// Hard limit in USD.
    pub limit_usd: f64,
    /// Amount spent so far.
    pub spent_usd: f64,
    /// Alert when spent >= limit * alert_threshold (0.0–1.0).
    pub alert_threshold: f64,
    /// Label for display.
    pub label: String,
}

impl BudgetEnvelope {
    pub fn new(label: impl Into<String>, limit_usd: f64, alert_threshold: f64) -> Self {
        Self {
            limit_usd,
            spent_usd: 0.0,
            alert_threshold: alert_threshold.clamp(0.0, 1.0),
            label: label.into(),
        }
    }

    /// Record a spend. Returns Err(BudgetExceeded) if hard limit is breached.
    pub fn spend(&mut self, amount_usd: f64) -> Result<(), DashboardError> {
        if amount_usd < 0.0 {
            return Err(DashboardError::Ledger("negative spend amount".into()));
        }
        self.spent_usd += amount_usd;
        if self.spent_usd > self.limit_usd {
            return Err(DashboardError::BudgetExceeded {
                spent: self.spent_usd,
                limit: self.limit_usd,
            });
        }
        Ok(())
    }

    /// Remaining budget in USD. Can go negative if over-limit.
    pub fn remaining(&self) -> f64 {
        self.limit_usd - self.spent_usd
    }

    /// Fraction consumed (0.0–1.0+).
    pub fn pct_consumed(&self) -> f64 {
        if self.limit_usd <= 0.0 {
            return 1.0;
        }
        self.spent_usd / self.limit_usd
    }

    /// Whether the hard limit has been exceeded.
    pub fn is_over_budget(&self) -> bool {
        self.spent_usd > self.limit_usd
    }

    /// Whether the alert threshold has been crossed.
    pub fn alert_triggered(&self) -> bool {
        self.pct_consumed() >= self.alert_threshold
    }

    /// Reset spend to zero.
    pub fn reset(&mut self) {
        self.spent_usd = 0.0;
    }

    /// ratatui gauge value: 0–100 clamped.
    pub fn gauge_pct(&self) -> u16 {
        (self.pct_consumed() * 100.0).clamp(0.0, 100.0) as u16
    }

    /// Traffic-light color label based on consumption.
    pub fn status(&self) -> &'static str {
        if self.pct_consumed() >= 1.0 {
            "OVER BUDGET"
        } else if self.alert_triggered() {
            "WARNING"
        } else {
            "OK"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_remaining_equals_limit() {
        let b = BudgetEnvelope::new("test", 10.0, 0.8);
        assert!((b.remaining() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_spend_decreases_remaining() {
        let mut b = BudgetEnvelope::new("test", 10.0, 0.8);
        b.spend(3.0).unwrap();
        assert!((b.remaining() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_over_budget_returns_error() {
        let mut b = BudgetEnvelope::new("test", 5.0, 0.8);
        assert!(b.spend(6.0).is_err());
    }

    #[test]
    fn test_over_budget_flag() {
        let mut b = BudgetEnvelope::new("test", 5.0, 0.8);
        let _ = b.spend(6.0);
        assert!(b.is_over_budget());
    }

    #[test]
    fn test_alert_not_triggered_below_threshold() {
        let b = BudgetEnvelope::new("test", 10.0, 0.8);
        assert!(!b.alert_triggered());
    }

    #[test]
    fn test_alert_triggered_at_threshold() {
        let mut b = BudgetEnvelope::new("test", 10.0, 0.8);
        b.spend(8.0).unwrap();
        assert!(b.alert_triggered());
    }

    #[test]
    fn test_reset_clears_spent() {
        let mut b = BudgetEnvelope::new("test", 10.0, 0.8);
        b.spend(5.0).unwrap();
        b.reset();
        assert_eq!(b.spent_usd, 0.0);
        assert!(!b.is_over_budget());
    }

    #[test]
    fn test_pct_consumed_zero_limit_returns_one() {
        let b = BudgetEnvelope::new("test", 0.0, 0.8);
        assert_eq!(b.pct_consumed(), 1.0);
    }

    #[test]
    fn test_gauge_pct_clamped_to_100() {
        let mut b = BudgetEnvelope::new("test", 5.0, 0.8);
        let _ = b.spend(10.0);
        assert_eq!(b.gauge_pct(), 100);
    }

    #[test]
    fn test_status_ok() {
        let b = BudgetEnvelope::new("test", 100.0, 0.8);
        assert_eq!(b.status(), "OK");
    }

    #[test]
    fn test_status_warning() {
        let mut b = BudgetEnvelope::new("test", 100.0, 0.8);
        b.spend(85.0).unwrap();
        assert_eq!(b.status(), "WARNING");
    }

    #[test]
    fn test_status_over_budget() {
        let mut b = BudgetEnvelope::new("test", 100.0, 0.8);
        let _ = b.spend(110.0);
        assert_eq!(b.status(), "OVER BUDGET");
    }

    #[test]
    fn test_negative_spend_returns_error() {
        let mut b = BudgetEnvelope::new("test", 100.0, 0.8);
        assert!(b.spend(-1.0).is_err());
    }
}
