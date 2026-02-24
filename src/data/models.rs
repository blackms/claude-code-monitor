use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct StatsCache {
    pub version: u32,
    pub last_computed_date: String,
    pub daily_activity: Vec<DailyActivity>,
    pub daily_model_tokens: Vec<DailyModelTokens>,
    pub model_usage: HashMap<String, ModelUsage>,
    pub total_sessions: u64,
    pub total_messages: u64,
    pub longest_session: Option<LongestSession>,
    pub first_session_date: Option<String>,
    pub hour_counts: HashMap<String, u64>,
    #[serde(default)]
    pub total_speculation_time_saved_ms: u64,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DailyActivity {
    pub date: String,
    pub message_count: u64,
    pub session_count: u64,
    pub tool_call_count: u64,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DailyModelTokens {
    pub date: String,
    pub tokens_by_model: HashMap<String, u64>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub web_search_requests: u64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub context_window: u64,
    #[serde(default)]
    pub max_output_tokens: u64,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LongestSession {
    pub session_id: String,
    pub duration: u64,
    pub message_count: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct HistoryEntry {
    pub display: String,
    #[serde(default)]
    pub pasted_contents: HashMap<String, String>,
    pub timestamp: u64,
    pub project: String,
    pub session_id: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionInfo {
    pub session_id: String,
    pub project: String,
    pub project_name: String,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub message_count: u64,
    pub is_active: bool,
}

impl SessionInfo {
    pub fn formatted_time(&self) -> String {
        use chrono::{Local, TimeZone};
        if let Some(dt) = Local
            .timestamp_millis_opt(self.last_timestamp as i64)
            .single()
        {
            dt.format("%b %d %H:%M").to_string()
        } else {
            "Unknown".to_string()
        }
    }
}
