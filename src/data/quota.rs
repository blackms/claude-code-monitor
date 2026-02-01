use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: OAuthTokens,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub scopes: Vec<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UsageResponse {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    pub seven_day_opus: Option<UsageWindow>,
    pub seven_day_sonnet: Option<UsageWindow>,
    pub seven_day_oauth_apps: Option<UsageWindow>,
    pub seven_day_cowork: Option<UsageWindow>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageWindow {
    pub utilization: f64,
    pub resets_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct QuotaInfo {
    pub session_usage: Option<f64>,
    pub session_resets_at: Option<String>,
    pub week_usage: Option<f64>,
    pub week_resets_at: Option<String>,
    pub sonnet_usage: Option<f64>,
    pub sonnet_resets_at: Option<String>,
    pub subscription_type: Option<String>,
    pub last_error: Option<String>,
    // Projection fields (calculated by UsageTracker)
    pub session_rate_per_hour: Option<f64>,
    pub week_rate_per_hour: Option<f64>,
    pub session_hours_to_depletion: Option<f64>,
    pub week_hours_to_depletion: Option<f64>,
    pub session_is_safe: Option<bool>,
    pub week_is_safe: Option<bool>,
    // Extra usage (pay-as-you-go overage)
    pub extra_usage_enabled: bool,
    pub extra_usage_monthly_limit: Option<f64>,
    pub extra_usage_used: Option<f64>,
    pub extra_usage_utilization: Option<f64>,
}

impl QuotaInfo {
    pub fn format_reset_time(iso_time: &str) -> String {
        // Parse ISO time and format nicely
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_time) {
            let local = dt.with_timezone(&chrono::Local);
            local.format("%b %d %H:%M").to_string()
        } else {
            iso_time.to_string()
        }
    }
}

/// Read OAuth credentials from macOS keychain
fn read_credentials() -> Result<Credentials> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to read credentials from keychain"));
    }

    let json_str = String::from_utf8(output.stdout)?;
    let creds: Credentials = serde_json::from_str(&json_str)?;
    Ok(creds)
}

/// Fetch usage quota from Anthropic API
pub fn fetch_quota() -> Result<QuotaInfo> {
    let creds = read_credentials()?;

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", creds.claude_ai_oauth.access_token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("API error: {}", response.status()));
    }

    let usage: UsageResponse = response.json()?;

    let mut quota = QuotaInfo {
        subscription_type: creds.claude_ai_oauth.subscription_type,
        ..Default::default()
    };

    if let Some(five_hour) = usage.five_hour {
        quota.session_usage = Some(five_hour.utilization);
        quota.session_resets_at = Some(five_hour.resets_at);
    }

    if let Some(seven_day) = usage.seven_day {
        quota.week_usage = Some(seven_day.utilization);
        quota.week_resets_at = Some(seven_day.resets_at);
    }

    if let Some(sonnet) = usage.seven_day_sonnet {
        quota.sonnet_usage = Some(sonnet.utilization);
        quota.sonnet_resets_at = Some(sonnet.resets_at);
    }

    // Extra usage (pay-as-you-go)
    // API returns values in cents, convert to dollars
    if let Some(extra) = usage.extra_usage {
        quota.extra_usage_enabled = extra.is_enabled;
        quota.extra_usage_monthly_limit = extra.monthly_limit.map(|v| v / 100.0);
        quota.extra_usage_used = extra.used_credits.map(|v| v / 100.0);
        quota.extra_usage_utilization = extra.utilization;
    }

    Ok(quota)
}
