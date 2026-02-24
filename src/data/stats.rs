use super::models::StatsCache;
use anyhow::Result;
use std::path::Path;

pub fn parse_stats_cache(path: &Path) -> Result<StatsCache> {
    let content = std::fs::read_to_string(path)?;
    let stats: StatsCache = serde_json::from_str(&content)?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_stats_cache() {
        let json = r#"{
            "version": 2,
            "lastComputedDate": "2026-01-31",
            "dailyActivity": [
                {"date": "2026-01-31", "messageCount": 100, "sessionCount": 5, "toolCallCount": 50}
            ],
            "dailyModelTokens": [],
            "modelUsage": {
                "claude-opus-4-5-20251101": {
                    "inputTokens": 1000,
                    "outputTokens": 500,
                    "cacheReadInputTokens": 10000,
                    "cacheCreationInputTokens": 5000,
                    "webSearchRequests": 0,
                    "costUSD": 0,
                    "contextWindow": 0,
                    "maxOutputTokens": 0
                }
            },
            "totalSessions": 10,
            "totalMessages": 1000,
            "longestSession": null,
            "firstSessionDate": "2026-01-01T00:00:00.000Z",
            "hourCounts": {"9": 5, "10": 3},
            "totalSpeculationTimeSavedMs": 0
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let stats = parse_stats_cache(file.path()).unwrap();
        assert_eq!(stats.version, 2);
        assert_eq!(stats.total_sessions, 10);
        assert_eq!(stats.total_messages, 1000);
        assert_eq!(stats.daily_activity.len(), 1);
    }
}
