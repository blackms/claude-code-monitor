use std::collections::HashMap;

use super::models::StatsCache;

/// Model pricing per million tokens (USD) - API pricing
#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_create: f64,
}

impl ModelPricing {
    pub fn for_model(model_name: &str) -> Self {
        if model_name.contains("opus") {
            // Claude Opus: $15/$75
            Self {
                input: 15.0,
                output: 75.0,
                cache_read: 1.50,
                cache_create: 18.75,
            }
        } else if model_name.contains("sonnet") {
            // Claude Sonnet 4 / 4.5: $3/$15
            Self {
                input: 3.0,
                output: 15.0,
                cache_read: 0.30,
                cache_create: 3.75,
            }
        } else if model_name.contains("haiku-4-5") {
            // Claude Haiku 4.5: $1/$5
            Self {
                input: 1.0,
                output: 5.0,
                cache_read: 0.10,
                cache_create: 1.25,
            }
        } else if model_name.contains("haiku-3-5") || model_name.contains("3-5-haiku") {
            // Claude Haiku 3.5: $0.80/$4
            Self {
                input: 0.80,
                output: 4.0,
                cache_read: 0.08,
                cache_create: 1.0,
            }
        } else if model_name.contains("haiku") {
            // Claude Haiku 3: $0.25/$1.25
            Self {
                input: 0.25,
                output: 1.25,
                cache_read: 0.03,
                cache_create: 0.30,
            }
        } else {
            // Default to sonnet pricing
            Self {
                input: 3.0,
                output: 15.0,
                cache_read: 0.30,
                cache_create: 3.75,
            }
        }
    }

    /// Calculate savings per million cache tokens vs full input pricing
    pub fn cache_savings_per_million(&self) -> f64 {
        self.input - self.cache_read
    }
}

/// Cache efficiency metrics
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CacheEfficiency {
    /// Overall hit ratio (cache_read / total_input)
    pub hit_ratio: f64,
    /// Total estimated savings in USD from cache
    pub savings_usd: f64,
    /// Hit ratio per model
    pub per_model: HashMap<String, f64>,
    /// Total cache read tokens
    pub total_cache_read: u64,
    /// Total input tokens (non-cached)
    pub total_input: u64,
}

impl CacheEfficiency {
    pub fn calculate(stats: &StatsCache) -> Self {
        let mut total_cache_read = 0u64;
        let mut total_input = 0u64;
        let mut total_savings = 0.0f64;
        let mut per_model = HashMap::new();

        for (model_name, usage) in &stats.model_usage {
            let pricing = ModelPricing::for_model(model_name);

            let cache_read = usage.cache_read_input_tokens;
            let input = usage.input_tokens;

            total_cache_read += cache_read;
            total_input += input;

            // Calculate hit ratio for this model
            let total_for_model = cache_read + input;
            if total_for_model > 0 {
                let ratio = cache_read as f64 / total_for_model as f64;
                per_model.insert(shorten_model_name(model_name), ratio * 100.0);
            }

            // Calculate savings: what we would have paid vs what we paid
            let savings = (cache_read as f64 / 1_000_000.0) * pricing.cache_savings_per_million();
            total_savings += savings;
        }

        let overall_total = total_cache_read + total_input;
        let hit_ratio = if overall_total > 0 {
            (total_cache_read as f64 / overall_total as f64) * 100.0
        } else {
            0.0
        };

        CacheEfficiency {
            hit_ratio,
            savings_usd: total_savings,
            per_model,
            total_cache_read,
            total_input,
        }
    }
}

/// Statistics averages
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Averages {
    pub messages_per_session: f64,
    pub tokens_per_message: f64,
}

impl Averages {
    pub fn calculate(stats: &StatsCache) -> Self {
        let messages_per_session = if stats.total_sessions > 0 {
            stats.total_messages as f64 / stats.total_sessions as f64
        } else {
            0.0
        };

        let total_tokens: u64 = stats
            .model_usage
            .values()
            .map(|u| u.input_tokens + u.output_tokens + u.cache_read_input_tokens)
            .sum();

        let tokens_per_message = if stats.total_messages > 0 {
            total_tokens as f64 / stats.total_messages as f64
        } else {
            0.0
        };

        Averages {
            messages_per_session,
            tokens_per_message,
        }
    }
}

/// Web search statistics
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WebSearchStats {
    pub total_searches: u64,
    pub searches_per_model: HashMap<String, u64>,
}

impl WebSearchStats {
    pub fn calculate(stats: &StatsCache) -> Self {
        let mut total = 0u64;
        let mut per_model = HashMap::new();

        for (model_name, usage) in &stats.model_usage {
            if usage.web_search_requests > 0 {
                total += usage.web_search_requests;
                per_model.insert(shorten_model_name(model_name), usage.web_search_requests);
            }
        }

        WebSearchStats {
            total_searches: total,
            searches_per_model: per_model,
        }
    }
}

/// Calculate monthly cost projection from recent daily average
pub fn calculate_monthly_projection(stats: &StatsCache) -> f64 {
    // Get the last 7 days of activity
    let recent_days: Vec<_> = stats.daily_activity.iter().rev().take(7).collect();

    if recent_days.is_empty() {
        return 0.0;
    }

    // Calculate average messages per day
    let total_messages: u64 = recent_days.iter().map(|d| d.message_count).sum();
    let avg_messages_per_day = total_messages as f64 / recent_days.len() as f64;

    // Estimate cost per message from total stats
    let total_cost = calculate_total_cost(stats);
    let cost_per_message = if stats.total_messages > 0 {
        total_cost / stats.total_messages as f64
    } else {
        0.0
    };

    // Project to 30 days
    avg_messages_per_day * cost_per_message * 30.0
}

/// Calculate total cost from all-time usage
pub fn calculate_total_cost(stats: &StatsCache) -> f64 {
    let mut total = 0.0;

    for (model_name, usage) in &stats.model_usage {
        let pricing = ModelPricing::for_model(model_name);

        total += (usage.input_tokens as f64 / 1_000_000.0) * pricing.input;
        total += (usage.output_tokens as f64 / 1_000_000.0) * pricing.output;
        
        // Worst-case calculation: Treat all cache operations as normal input tokens
        total += (usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.input;
        total += (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * pricing.input;
    }

    total
}

/// Format a duration in milliseconds to human readable
pub fn format_duration_ms(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Format a date string for display
pub fn format_first_session_date(date_str: &str) -> String {
    // Try parsing ISO format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return dt.format("%b %d, %Y").to_string();
    }

    // Try parsing date only format
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return date.format("%b %d, %Y").to_string();
    }

    date_str.to_string()
}

/// Shorten model name for display
pub fn shorten_model_name(name: &str) -> String {
    if name.contains("opus-4-6") {
        "opus-4-6".to_string()
    } else if name.contains("opus-4-5") {
        "opus-4-5".to_string()
    } else if name.contains("opus-4-1") {
        "opus-4-1".to_string()
    } else if name.contains("opus-4") {
        "opus-4".to_string()
    } else if name.contains("sonnet-4-5") {
        "sonnet-4-5".to_string()
    } else if name.contains("sonnet-4") {
        "sonnet-4".to_string()
    } else if name.contains("haiku-4-5") {
        "haiku-4-5".to_string()
    } else if name.contains("haiku-3-5") || name.contains("3-5-haiku") {
        "haiku-3-5".to_string()
    } else if name.contains("haiku") {
        "haiku".to_string()
    } else {
        name.chars().take(12).collect()
    }
}

/// Format number with thousand separators
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format currency value
pub fn format_currency(value: f64) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1).unwrap_or(&"00");

    let mut result = String::new();
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    format!(
        "${}.{}",
        result.chars().rev().collect::<String>(),
        decimal_part
    )
}

/// Format tokens in human readable form (K, M)
#[allow(dead_code)]
pub fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.0}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing() {
        let opus46 = ModelPricing::for_model("claude-opus-4-6-20260101");
        assert_eq!(opus46.input, 15.0);
        assert_eq!(opus46.output, 75.0);

        let opus45 = ModelPricing::for_model("claude-opus-4-5-20251101");
        assert_eq!(opus45.input, 15.0);
        assert_eq!(opus45.output, 75.0);

        let opus4 = ModelPricing::for_model("claude-opus-4-20250101");
        assert_eq!(opus4.input, 15.0);

        let sonnet = ModelPricing::for_model("sonnet-4-5-20251202");
        assert_eq!(sonnet.input, 3.0);

        let haiku45 = ModelPricing::for_model("claude-haiku-4-5-20251001");
        assert_eq!(haiku45.input, 1.0);
        assert_eq!(haiku45.output, 5.0);

        let haiku35 = ModelPricing::for_model("claude-3-5-haiku-20241022");
        assert_eq!(haiku35.input, 0.80);

        let haiku3 = ModelPricing::for_model("claude-3-haiku-20240307");
        assert_eq!(haiku3.input, 0.25);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration_ms(1800000), "30m");
        assert_eq!(format_duration_ms(5400000), "1h 30m");
        assert_eq!(format_duration_ms(9000000), "2h 30m");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(1234.56), "$1,234.56");
        assert_eq!(format_currency(0.99), "$0.99");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1500), "2K");
        assert_eq!(format_tokens(1500000), "1.5M");
    }

    #[test]
    fn test_cache_savings() {
        let opus = ModelPricing::for_model("opus-4-1");
        assert!((opus.cache_savings_per_million() - 13.5).abs() < 0.01);

        let opus45 = ModelPricing::for_model("opus-4-5");
        assert!((opus45.cache_savings_per_million() - 13.5).abs() < 0.01);

        let sonnet = ModelPricing::for_model("sonnet");
        assert!((sonnet.cache_savings_per_million() - 2.7).abs() < 0.01);
    }
}
