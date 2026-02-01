use chrono::{DateTime, Datelike, Local, NaiveDate};

use super::models::{DailyActivity, HistoryEntry};

/// Direction of a trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    Flat,
    NoData,
}

impl Trend {
    pub fn symbol(&self) -> &'static str {
        match self {
            Trend::Up => "▲",
            Trend::Down => "▼",
            Trend::Flat => "─",
            Trend::NoData => "?",
        }
    }
}

/// Comparison between two periods
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PeriodComparison {
    pub current_messages: u64,
    pub previous_messages: u64,
    pub current_tokens: u64,
    pub previous_tokens: u64,
    pub current_cost: f64,
    pub previous_cost: f64,
    pub messages_trend: Trend,
    pub tokens_trend: Trend,
    pub cost_trend: Trend,
    pub messages_change_pct: Option<f64>,
    pub tokens_change_pct: Option<f64>,
    pub cost_change_pct: Option<f64>,
}

impl Default for PeriodComparison {
    fn default() -> Self {
        Self {
            current_messages: 0,
            previous_messages: 0,
            current_tokens: 0,
            previous_tokens: 0,
            current_cost: 0.0,
            previous_cost: 0.0,
            messages_trend: Trend::NoData,
            tokens_trend: Trend::NoData,
            cost_trend: Trend::NoData,
            messages_change_pct: None,
            tokens_change_pct: None,
            cost_change_pct: None,
        }
    }
}

/// Aggregated trend data
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TrendData {
    pub day: PeriodComparison,      // Today vs Yesterday
    pub week: PeriodComparison,     // This week vs Last week
    pub month: PeriodComparison,    // This month vs Last month
    pub sparkline: Vec<u64>,        // Last 7 days message counts for sparkline
}

impl TrendData {
    /// Calculate trends from history entries and daily activity
    pub fn calculate(entries: &[HistoryEntry], daily_activity: &[DailyActivity]) -> Self {
        let now = Local::now();
        let today = now.date_naive();
        let yesterday = today.pred_opt().unwrap_or(today);

        // Calculate day comparison from history entries (more accurate for today)
        let day = Self::calculate_day_comparison(entries, today, yesterday);

        // Calculate week comparison
        let week = Self::calculate_week_comparison(daily_activity, today);

        // Calculate month comparison
        let month = Self::calculate_month_comparison(daily_activity, today);

        // Build sparkline from last 7 days
        let sparkline = Self::build_sparkline(daily_activity, today, 7);

        TrendData {
            day,
            week,
            month,
            sparkline,
        }
    }

    fn calculate_day_comparison(
        entries: &[HistoryEntry],
        today: NaiveDate,
        yesterday: NaiveDate,
    ) -> PeriodComparison {
        let mut today_messages = 0u64;
        let mut yesterday_messages = 0u64;

        for entry in entries {
            if let Some(dt) = DateTime::from_timestamp_millis(entry.timestamp as i64) {
                let entry_date = dt.with_timezone(&Local).date_naive();
                if entry_date == today {
                    today_messages += 1;
                } else if entry_date == yesterday {
                    yesterday_messages += 1;
                }
            }
        }

        Self::build_comparison(
            today_messages,
            yesterday_messages,
            0, 0,  // Tokens not available from history entries
            0.0, 0.0,  // Cost not available
        )
    }

    fn calculate_week_comparison(
        daily_activity: &[DailyActivity],
        today: NaiveDate,
    ) -> PeriodComparison {
        // This week: from Monday of current week to today
        // Last week: same days but previous week
        let days_since_monday = today.weekday().num_days_from_monday() as i64;
        let this_week_start = today - chrono::Duration::days(days_since_monday);
        let last_week_start = this_week_start - chrono::Duration::days(7);
        let last_week_end = this_week_start - chrono::Duration::days(1);

        let mut this_week_messages = 0u64;
        let mut last_week_messages = 0u64;

        for activity in daily_activity {
            if let Ok(date) = NaiveDate::parse_from_str(&activity.date, "%Y-%m-%d") {
                if date >= this_week_start && date <= today {
                    this_week_messages += activity.message_count;
                } else if date >= last_week_start && date <= last_week_end {
                    last_week_messages += activity.message_count;
                }
            }
        }

        Self::build_comparison(
            this_week_messages,
            last_week_messages,
            0, 0,
            0.0, 0.0,
        )
    }

    fn calculate_month_comparison(
        daily_activity: &[DailyActivity],
        today: NaiveDate,
    ) -> PeriodComparison {
        // Last 30 days vs prior 30 days
        let period_start = today - chrono::Duration::days(29);
        let prev_period_end = period_start - chrono::Duration::days(1);
        let prev_period_start = prev_period_end - chrono::Duration::days(29);

        let mut current_messages = 0u64;
        let mut previous_messages = 0u64;

        for activity in daily_activity {
            if let Ok(date) = NaiveDate::parse_from_str(&activity.date, "%Y-%m-%d") {
                if date >= period_start && date <= today {
                    current_messages += activity.message_count;
                } else if date >= prev_period_start && date <= prev_period_end {
                    previous_messages += activity.message_count;
                }
            }
        }

        Self::build_comparison(
            current_messages,
            previous_messages,
            0, 0,
            0.0, 0.0,
        )
    }

    fn build_sparkline(
        daily_activity: &[DailyActivity],
        today: NaiveDate,
        days: i64,
    ) -> Vec<u64> {
        let mut counts = vec![0u64; days as usize];

        for activity in daily_activity {
            if let Ok(date) = NaiveDate::parse_from_str(&activity.date, "%Y-%m-%d") {
                let days_ago = (today - date).num_days();
                if days_ago >= 0 && days_ago < days {
                    counts[(days - 1 - days_ago) as usize] = activity.message_count;
                }
            }
        }

        counts
    }

    fn build_comparison(
        current_messages: u64,
        previous_messages: u64,
        current_tokens: u64,
        previous_tokens: u64,
        current_cost: f64,
        previous_cost: f64,
    ) -> PeriodComparison {
        let (messages_trend, messages_change_pct) =
            Self::calculate_trend(current_messages as f64, previous_messages as f64);
        let (tokens_trend, tokens_change_pct) =
            Self::calculate_trend(current_tokens as f64, previous_tokens as f64);
        let (cost_trend, cost_change_pct) =
            Self::calculate_trend(current_cost, previous_cost);

        PeriodComparison {
            current_messages,
            previous_messages,
            current_tokens,
            previous_tokens,
            current_cost,
            previous_cost,
            messages_trend,
            tokens_trend,
            cost_trend,
            messages_change_pct,
            tokens_change_pct,
            cost_change_pct,
        }
    }

    fn calculate_trend(current: f64, previous: f64) -> (Trend, Option<f64>) {
        if previous < 0.001 && current < 0.001 {
            return (Trend::NoData, None);
        }

        if previous < 0.001 {
            return (Trend::Up, None); // Can't calculate percentage from zero
        }

        let change_pct = ((current - previous) / previous) * 100.0;

        let trend = if change_pct.abs() < 1.0 {
            Trend::Flat
        } else if change_pct > 0.0 {
            Trend::Up
        } else {
            Trend::Down
        };

        (trend, Some(change_pct))
    }
}

/// Render sparkline from data points
pub fn render_sparkline(data: &[u64]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = *data.iter().max().unwrap_or(&1).max(&1);

    data.iter()
        .map(|&value| {
            let idx = if max > 0 {
                ((value as f64 / max as f64) * 7.0).round() as usize
            } else {
                0
            };
            bars[idx.min(7)]
        })
        .collect()
}

/// Format percentage change for display
#[allow(dead_code)]
pub fn format_change(pct: Option<f64>, trend: Trend) -> String {
    match pct {
        Some(p) => format!("{} ({:+.0}%)", trend.symbol(), p),
        None => trend.symbol().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_symbols() {
        assert_eq!(Trend::Up.symbol(), "▲");
        assert_eq!(Trend::Down.symbol(), "▼");
        assert_eq!(Trend::Flat.symbol(), "─");
    }

    #[test]
    fn test_render_sparkline() {
        let data = vec![1, 2, 4, 3, 5, 7, 8];
        let result = render_sparkline(&data);
        assert_eq!(result.chars().count(), 7);
    }

    #[test]
    fn test_format_change() {
        assert_eq!(format_change(Some(25.0), Trend::Up), "▲ (+25%)");
        assert_eq!(format_change(Some(-10.0), Trend::Down), "▼ (-10%)");
        assert_eq!(format_change(None, Trend::NoData), "?");
    }
}
