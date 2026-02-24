pub mod calculations;
pub mod history;
pub mod models;
pub mod persistence;
pub mod quota;
pub mod stats;
pub mod trends;
pub mod usage_tracker;

pub use calculations::{
    calculate_monthly_projection, calculate_total_cost, format_currency, format_duration_ms,
    format_first_session_date, format_number, shorten_model_name, Averages, CacheEfficiency,
    ModelPricing, WebSearchStats,
};
pub use history::{
    count_projects, count_recent_messages, count_today_messages, group_sessions, parse_history,
    sync_history_archive, ProjectStats,
};
pub use models::*;
pub use quota::{fetch_quota, QuotaInfo};
pub use stats::parse_stats_cache;
pub use trends::{render_sparkline, Trend, TrendData};
pub use usage_tracker::{format_hours, DepletionStatus, UsageTracker};
