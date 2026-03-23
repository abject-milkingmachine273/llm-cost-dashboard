//! # Spend Forecast Engine
//!
//! Projects cumulative LLM spend to the end of the calendar month using
//! Ordinary Least Squares (OLS) linear regression over a series of
//! `(timestamp_secs, cumulative_cost_usd)` observations.
//!
//! ## Example
//!
//! ```
//! use llm_cost_dashboard::forecast::SpendForecaster;
//!
//! let mut f = SpendForecaster::new();
//! // Simulate 3 days of spend observations.
//! let base: f64 = 1_700_000_000.0;
//! f.record(base,             0.00);
//! f.record(base + 86_400.0,  5.50);
//! f.record(base + 172_800.0, 11.00);
//!
//! if let Some(result) = f.forecast(Some(100.0)) {
//!     println!("projected month-end: ${:.2}", result.projected_month_end_usd);
//! }
//! ```

/// The directional trend of spend derived from regression slope changes.
#[derive(Debug, Clone, PartialEq)]
pub enum Trend {
    /// Spend rate is increasing (second-half slope > first-half slope).
    Accelerating,
    /// Spend rate is roughly constant.
    Stable,
    /// Spend rate is slowing down (second-half slope < first-half slope).
    Decelerating,
}

/// The output of a spend forecast computation.
#[derive(Debug, Clone)]
pub struct ForecastResult {
    /// Projected cumulative USD spend at midnight on the last day of the
    /// current calendar month.
    pub projected_month_end_usd: f64,
    /// Projected average daily spend in USD (regression slope × 86 400 s/day).
    pub projected_daily_usd: f64,
    /// Number of days until the budget limit is hit based on the current
    /// regression slope.  `None` when no budget limit was supplied or spend
    /// is already at/above the limit.
    pub days_until_budget_hit: Option<f64>,
    /// Goodness-of-fit confidence in `[0.0, 1.0]` derived from the R²
    /// coefficient of determination.  Values close to `1.0` indicate the
    /// linear model fits the data well.
    pub confidence: f64,
    /// Directional trend inferred by comparing the slope of the first half of
    /// observations against the second half.
    pub trend: Trend,
}

/// Linear-regression spend forecaster.
///
/// Records a time series of `(unix_timestamp_secs, cumulative_cost_usd)` pairs
/// and fits an OLS line to project spend forward to the end of the month.
///
/// At least two distinct observations are required before [`forecast`] can
/// return a result.
///
/// [`forecast`]: SpendForecaster::forecast
pub struct SpendForecaster {
    /// Stored `(timestamp_secs, cumulative_cost_usd)` pairs, sorted by
    /// insertion order (callers are expected to insert chronologically).
    observations: Vec<(f64, f64)>,
}

impl Default for SpendForecaster {
    fn default() -> Self {
        Self::new()
    }
}

impl SpendForecaster {
    /// Create an empty forecaster.
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    /// Append a `(timestamp_secs, cumulative_cost_usd)` observation.
    ///
    /// Observations should be supplied in chronological order.  Internally no
    /// sorting is performed; out-of-order data will produce inaccurate trend
    /// classification but the regression itself is still mathematically valid.
    pub fn record(&mut self, timestamp_secs: f64, cumulative_cost: f64) {
        self.observations.push((timestamp_secs, cumulative_cost));
    }

    /// Number of observations recorded so far.
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// Whether no observations have been recorded yet.
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Project spend to the end of the current calendar month using OLS linear
    /// regression.
    ///
    /// Returns `None` when fewer than two observations are available (regression
    /// is undefined).
    ///
    /// # Arguments
    ///
    /// * `budget_limit` – optional monthly budget cap in USD.  When supplied,
    ///   [`ForecastResult::days_until_budget_hit`] is populated.
    pub fn forecast(&self, budget_limit: Option<f64>) -> Option<ForecastResult> {
        if self.observations.len() < 2 {
            return None;
        }

        let (slope, intercept) = self.linear_regression()?;

        // Seconds per day.
        let spd = 86_400.0_f64;

        // Projected daily spend (USD/day) from the slope.
        let projected_daily_usd = slope * spd;

        // Determine seconds until end-of-month from the most recent timestamp.
        let last_ts = self.observations.last().map(|(t, _)| *t).unwrap_or(0.0);
        let secs_to_month_end = seconds_to_month_end(last_ts);

        // Project cumulative cost at month end.
        let last_cost = self.observations.last().map(|(_, c)| *c).unwrap_or(0.0);
        let projected_month_end_usd = last_cost + slope * secs_to_month_end;

        // Days until budget is hit.
        let days_until_budget_hit = budget_limit.and_then(|limit| {
            if last_cost >= limit || slope <= 0.0 {
                None
            } else {
                let secs_remaining = (limit - last_cost) / slope;
                Some(secs_remaining / spd)
            }
        });

        // R² confidence.
        let confidence = self.r_squared(slope, intercept);

        // Trend: compare slope of first half vs second half.
        let trend = self.classify_trend();

        Some(ForecastResult {
            projected_month_end_usd,
            projected_daily_usd,
            days_until_budget_hit,
            confidence,
            trend,
        })
    }

    /// Perform OLS linear regression on the stored observations.
    ///
    /// Returns `(slope, intercept)` where `slope` is in USD per second and
    /// `intercept` is in USD.  Returns `None` when the observations are
    /// collinear in the x-axis (all timestamps identical).
    fn linear_regression(&self) -> Option<(f64, f64)> {
        let n = self.observations.len() as f64;
        let sum_x: f64 = self.observations.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = self.observations.iter().map(|(_, y)| y).sum();
        let sum_xx: f64 = self.observations.iter().map(|(x, _)| x * x).sum();
        let sum_xy: f64 = self.observations.iter().map(|(x, y)| x * y).sum();

        let denom = n * sum_xx - sum_x * sum_x;
        if denom.abs() < f64::EPSILON {
            return None;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        Some((slope, intercept))
    }

    /// Compute R² (coefficient of determination) for the given regression line.
    ///
    /// Returns a value in `[0.0, 1.0]`, clamped to handle floating-point
    /// edge cases.
    fn r_squared(&self, slope: f64, intercept: f64) -> f64 {
        let n = self.observations.len() as f64;
        let mean_y: f64 = self.observations.iter().map(|(_, y)| y).sum::<f64>() / n;

        let ss_tot: f64 = self
            .observations
            .iter()
            .map(|(_, y)| (y - mean_y).powi(2))
            .sum();

        if ss_tot < f64::EPSILON {
            // All y values are identical; perfect fit by convention.
            return 1.0;
        }

        let ss_res: f64 = self
            .observations
            .iter()
            .map(|(x, y)| {
                let predicted = slope * x + intercept;
                (y - predicted).powi(2)
            })
            .sum();

        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    }

    /// Classify spend trend by comparing the OLS slope of the first half of
    /// observations against the second half.
    ///
    /// Falls back to [`Trend::Stable`] when either half has fewer than two
    /// points or regression is undefined.
    fn classify_trend(&self) -> Trend {
        let n = self.observations.len();
        let mid = n / 2;

        let slope_first = {
            let half = SpendForecaster {
                observations: self.observations[..mid].to_vec(),
            };
            half.linear_regression().map(|(s, _)| s)
        };

        let slope_second = {
            let half = SpendForecaster {
                observations: self.observations[mid..].to_vec(),
            };
            half.linear_regression().map(|(s, _)| s)
        };

        match (slope_first, slope_second) {
            (Some(s1), Some(s2)) => {
                // Allow a 10% band around stable.
                let ratio = if s1.abs() > f64::EPSILON { s2 / s1 } else { 1.0 };
                if ratio > 1.10 {
                    Trend::Accelerating
                } else if ratio < 0.90 {
                    Trend::Decelerating
                } else {
                    Trend::Stable
                }
            }
            _ => Trend::Stable,
        }
    }
}

/// Compute the number of seconds remaining until midnight on the last day of
/// the calendar month that contains `unix_ts`.
///
/// Uses a simple leap-year-aware month-length table.  The result is always
/// `>= 0.0`.
fn seconds_to_month_end(unix_ts: f64) -> f64 {
    // Days since Unix epoch.
    let days_since_epoch = (unix_ts / 86_400.0).floor() as i64;

    // Compute year/month/day from days-since-epoch using the proleptic
    // Gregorian calendar algorithm (Fliegel & Van Flandern).
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    let days_in_month = month_days(year, m as u32);

    // Seconds elapsed within the current month.
    let secs_today = unix_ts % 86_400.0;
    let day_of_month = {
        let d = doy - (153 * mp + 2) / 5;
        d + 1 // 1-based
    };

    let days_remaining = days_in_month as i64 - day_of_month;
    (days_remaining as f64 * 86_400.0 + (86_400.0 - secs_today)).max(0.0)
}

/// Number of days in the given (year, month) pair (month is 1-based).
fn month_days(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Returns `true` for proleptic Gregorian leap years.
fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_forecaster_linear(n: usize, slope_per_day: f64) -> SpendForecaster {
        let mut f = SpendForecaster::new();
        let base: f64 = 1_700_000_000.0; // 2023-11-14-ish
        let spd = 86_400.0;
        for i in 0..n {
            let t = base + i as f64 * spd;
            let cost = i as f64 * slope_per_day;
            f.record(t, cost);
        }
        f
    }

    #[test]
    fn test_new_is_empty() {
        let f = SpendForecaster::new();
        assert!(f.is_empty());
        assert_eq!(f.len(), 0);
    }

    #[test]
    fn test_record_increases_len() {
        let mut f = SpendForecaster::new();
        f.record(1_700_000_000.0, 0.0);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn test_forecast_requires_two_observations() {
        let mut f = SpendForecaster::new();
        f.record(1_700_000_000.0, 0.0);
        assert!(f.forecast(None).is_none());
    }

    #[test]
    fn test_forecast_returns_some_with_two_points() {
        let mut f = SpendForecaster::new();
        f.record(1_700_000_000.0, 0.0);
        f.record(1_700_086_400.0, 5.0);
        assert!(f.forecast(None).is_some());
    }

    #[test]
    fn test_daily_projection_matches_slope() {
        // $5/day linear spend.
        let f = make_forecaster_linear(10, 5.0);
        let result = f.forecast(None).unwrap();
        assert!((result.projected_daily_usd - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_high_r_squared_for_perfect_linear_data() {
        let f = make_forecaster_linear(20, 10.0);
        let result = f.forecast(None).unwrap();
        // Perfect linear data should yield R² ≈ 1.0.
        assert!(result.confidence > 0.99);
    }

    #[test]
    fn test_days_until_budget_hit_none_when_no_limit() {
        let f = make_forecaster_linear(10, 5.0);
        let result = f.forecast(None).unwrap();
        assert!(result.days_until_budget_hit.is_none());
    }

    #[test]
    fn test_days_until_budget_hit_some_when_under_limit() {
        // $5/day, already at $45, limit = $100.  Need $55 more => ~11 days.
        let f = make_forecaster_linear(10, 5.0);
        let result = f.forecast(Some(100.0)).unwrap();
        let days = result.days_until_budget_hit.unwrap();
        assert!((days - 11.0).abs() < 0.5);
    }

    #[test]
    fn test_days_until_budget_hit_none_when_over_limit() {
        // Limit below current spend.
        let f = make_forecaster_linear(10, 5.0);
        let result = f.forecast(Some(1.0)).unwrap();
        assert!(result.days_until_budget_hit.is_none());
    }

    #[test]
    fn test_trend_stable_for_constant_slope() {
        let f = make_forecaster_linear(20, 5.0);
        let result = f.forecast(None).unwrap();
        assert_eq!(result.trend, Trend::Stable);
    }

    #[test]
    fn test_trend_accelerating() {
        let mut f = SpendForecaster::new();
        let base = 1_700_000_000.0;
        let spd = 86_400.0;
        // First half: slow spend (slope ≈ 1/day).
        for i in 0..10 {
            f.record(base + i as f64 * spd, i as f64 * 1.0);
        }
        // Second half: much faster spend (slope ≈ 10/day).
        for i in 10..20 {
            f.record(base + i as f64 * spd, 10.0 + (i - 10) as f64 * 10.0);
        }
        let result = f.forecast(None).unwrap();
        assert_eq!(result.trend, Trend::Accelerating);
    }

    #[test]
    fn test_r_squared_method() {
        let f = make_forecaster_linear(15, 3.0);
        let (slope, intercept) = f.linear_regression().unwrap();
        let r2 = f.r_squared(slope, intercept);
        assert!(r2 > 0.99);
    }

    #[test]
    fn test_projected_month_end_positive() {
        let f = make_forecaster_linear(5, 2.0);
        let result = f.forecast(None).unwrap();
        assert!(result.projected_month_end_usd >= 0.0);
    }

    #[test]
    fn test_default_impl() {
        let f = SpendForecaster::default();
        assert!(f.is_empty());
    }
}
