use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Sample of quota utilization at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSample {
    pub timestamp: DateTime<Utc>,
    pub session_utilization: Option<f64>,
    pub week_utilization: Option<f64>,
}

/// Tracks quota usage over time for projection calculations
#[derive(Debug, Clone)]
pub struct UsageTracker {
    /// Ring buffer of recent samples (max 180 samples = ~6 minutes at 2s intervals)
    samples: VecDeque<QuotaSample>,
    max_samples: usize,
}

/// Status of quota depletion
#[derive(Debug, Clone)]
pub enum DepletionStatus {
    /// Quota will be depleted before reset
    Depleting { hours_remaining: f64 },
    /// Quota will reset before depletion
    Safe { hours_until_reset: f64 },
    /// No usage detected
    NoUsage,
    /// Not enough data to calculate
    Unknown,
}

impl DepletionStatus {
    #[allow(dead_code)]
    pub fn is_warning(&self) -> bool {
        matches!(self, DepletionStatus::Depleting { .. })
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new(180)
    }
}

impl UsageTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Add a new quota sample
    pub fn add_sample(&mut self, session_util: Option<f64>, week_util: Option<f64>) {
        let sample = QuotaSample {
            timestamp: Utc::now(),
            session_utilization: session_util,
            week_utilization: week_util,
        };

        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Calculate the usage rate per hour using linear regression
    /// Returns None if not enough data
    pub fn calculate_rate_per_hour(&self, use_session: bool) -> Option<f64> {
        let data: Vec<(f64, f64)> = self
            .samples
            .iter()
            .filter_map(|s| {
                let util = if use_session {
                    s.session_utilization?
                } else {
                    s.week_utilization?
                };
                Some((s.timestamp.timestamp() as f64, util))
            })
            .collect();

        if data.len() < 3 {
            return None;
        }

        // Linear regression: y = mx + b
        // We want m (slope) in units of percent per hour
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < f64::EPSILON {
            return None;
        }

        // Slope is in percent per second
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;

        // Convert to percent per hour
        let rate_per_hour = slope * 3600.0;

        // Only return positive rates (usage increasing)
        if rate_per_hour > 0.001 {
            Some(rate_per_hour)
        } else {
            Some(0.0)
        }
    }

    /// Calculate depletion status for session quota
    pub fn session_depletion_status(&self, current_util: f64, resets_at: &str) -> DepletionStatus {
        self.calculate_depletion(current_util, resets_at, true)
    }

    /// Calculate depletion status for week quota
    pub fn week_depletion_status(&self, current_util: f64, resets_at: &str) -> DepletionStatus {
        self.calculate_depletion(current_util, resets_at, false)
    }

    fn calculate_depletion(
        &self,
        current_util: f64,
        resets_at: &str,
        use_session: bool,
    ) -> DepletionStatus {
        // Parse reset time
        let reset_time = match DateTime::parse_from_rfc3339(resets_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return DepletionStatus::Unknown,
        };

        let now = Utc::now();
        let hours_until_reset = (reset_time - now).num_seconds() as f64 / 3600.0;

        if hours_until_reset <= 0.0 {
            return DepletionStatus::Unknown;
        }

        // Get rate per hour
        let rate = match self.calculate_rate_per_hour(use_session) {
            Some(r) if r > 0.001 => r,
            _ => return DepletionStatus::NoUsage,
        };

        // Calculate hours to depletion
        let remaining_percent = 100.0 - current_util;
        if remaining_percent <= 0.0 {
            return DepletionStatus::Depleting {
                hours_remaining: 0.0,
            };
        }

        let hours_to_depletion = remaining_percent / rate;

        if hours_to_depletion < hours_until_reset {
            DepletionStatus::Depleting {
                hours_remaining: hours_to_depletion,
            }
        } else {
            DepletionStatus::Safe { hours_until_reset }
        }
    }

    /// Get the number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Check if we have enough samples for reliable projections
    pub fn has_enough_data(&self) -> bool {
        self.samples.len() >= 3
    }

    /// Get a reference to all samples (for persistence)
    pub fn get_samples(&self) -> &VecDeque<QuotaSample> {
        &self.samples
    }

    /// Add a sample with a specific timestamp (for loading from persistence)
    pub fn add_sample_with_timestamp(
        &mut self,
        timestamp: DateTime<Utc>,
        session_util: Option<f64>,
        week_util: Option<f64>,
    ) {
        let sample = QuotaSample {
            timestamp,
            session_utilization: session_util,
            week_utilization: week_util,
        };

        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }
}

/// Format hours into a human-readable string
pub fn format_hours(hours: f64) -> String {
    if hours < 1.0 {
        let minutes = (hours * 60.0).round() as i32;
        format!("{}m", minutes)
    } else if hours < 24.0 {
        format!("{:.1}h", hours)
    } else {
        let days = hours / 24.0;
        format!("{:.1}d", days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_tracker_add_sample() {
        let mut tracker = UsageTracker::new(10);
        tracker.add_sample(Some(50.0), Some(25.0));
        assert_eq!(tracker.sample_count(), 1);
    }

    #[test]
    fn test_usage_tracker_max_samples() {
        let mut tracker = UsageTracker::new(3);
        tracker.add_sample(Some(10.0), None);
        tracker.add_sample(Some(20.0), None);
        tracker.add_sample(Some(30.0), None);
        tracker.add_sample(Some(40.0), None);
        assert_eq!(tracker.sample_count(), 3);
    }

    #[test]
    fn test_format_hours() {
        assert_eq!(format_hours(0.5), "30m");
        assert_eq!(format_hours(2.5), "2.5h");
        assert_eq!(format_hours(48.0), "2.0d");
    }
}
