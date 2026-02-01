pub mod models;
pub mod stats;
pub mod history;
pub mod quota;
pub mod usage_tracker;
pub mod trends;
pub mod calculations;
pub mod persistence;

pub use models::*;
pub use stats::parse_stats_cache;
pub use history::{parse_history, group_sessions, count_today_messages, count_recent_messages, count_projects, ProjectStats};
pub use quota::{fetch_quota, QuotaInfo};
pub use usage_tracker::{UsageTracker, DepletionStatus, format_hours};
pub use trends::{TrendData, Trend, render_sparkline};
pub use calculations::{
    CacheEfficiency, Averages, WebSearchStats,
    calculate_monthly_projection,
    format_duration_ms, format_first_session_date,
    format_number, format_currency,
};
