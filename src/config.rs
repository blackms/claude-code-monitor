use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

/// User-facing configuration file format (~/.config/claude-code-monitor/config.toml)

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreset {
    #[default]
    Default,
    Catppuccin,
    Dracula,
    Nord,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigFile {
    /// UI refresh rate in milliseconds (default: 250)
    pub refresh_rate_ms: Option<u64>,
    /// Quota polling interval in seconds (default: 2)
    pub quota_refresh_secs: Option<u64>,
    /// Stats/history polling interval in seconds (default: 10)
    pub data_refresh_secs: Option<u64>,
    /// Path to Claude Code data directory (default: ~/.claude)
    pub claude_dir: Option<String>,
    /// UI Theme (default, catppuccin, dracula, nord)
    pub theme: Option<ThemePreset>,
}

#[allow(dead_code)]
pub struct Config {
    pub claude_dir: PathBuf,
    pub stats_file: PathBuf,
    pub history_file: PathBuf,
    pub archive_file: PathBuf,
    pub samples_file: PathBuf,
    pub refresh_rate: Duration,
    pub quota_refresh_rate: Duration,
    pub data_refresh_rate: Duration,
    pub theme: ThemePreset,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        // Try to load config file
        let config_file = Self::load_config_file(&home);

        // Determine claude_dir
        let claude_dir = if let Some(ref dir) = config_file.claude_dir {
            PathBuf::from(shellexpand(&dir, &home))
        } else {
            home.join(".claude")
        };

        Ok(Self {
            stats_file: claude_dir.join("stats-cache.json"),
            history_file: claude_dir.join("history.jsonl"),
            archive_file: claude_dir.join("history-archive.jsonl"),
            samples_file: claude_dir.join("usage-samples.json"),
            claude_dir,
            theme: config_file.theme.unwrap_or_default(),
            refresh_rate: Duration::from_millis(config_file.refresh_rate_ms.unwrap_or(250)),
            quota_refresh_rate: Duration::from_secs(config_file.quota_refresh_secs.unwrap_or(2)),
            data_refresh_rate: Duration::from_secs(config_file.data_refresh_secs.unwrap_or(10)),
        })
    }

    /// Try to load the config file from standard locations
    fn load_config_file(home: &PathBuf) -> ConfigFile {
        // Try ~/.config/claude-code-monitor/config.toml
        let config_path = home
            .join(".config")
            .join("claude-code-monitor")
            .join("config.toml");

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<ConfigFile>(&content) {
                return config;
            }
        }

        ConfigFile::default()
    }
}

/// Simple ~ expansion for paths in config
fn shellexpand(path: &str, home: &PathBuf) -> String {
    if path.starts_with("~/") {
        home.join(&path[2..]).to_string_lossy().to_string()
    } else {
        path.to_string()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Failed to create default config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::new().unwrap();
        assert_eq!(config.refresh_rate, Duration::from_millis(250));
        assert_eq!(config.quota_refresh_rate, Duration::from_secs(2));
        assert_eq!(config.data_refresh_rate, Duration::from_secs(10));
    }

    #[test]
    fn test_config_file_parse() {
        let toml_str = r#"
            refresh_rate_ms = 500
            quota_refresh_secs = 5
            data_refresh_secs = 30
            claude_dir = "~/.claude"
        "#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.refresh_rate_ms, Some(500));
        assert_eq!(config.quota_refresh_secs, Some(5));
        assert_eq!(config.data_refresh_secs, Some(30));
    }

    #[test]
    fn test_config_file_partial() {
        let toml_str = r#"
            refresh_rate_ms = 300
        "#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.refresh_rate_ms, Some(300));
        assert_eq!(config.quota_refresh_secs, None);
    }

    #[test]
    fn test_shellexpand() {
        let home = PathBuf::from("/Users/test");
        assert_eq!(shellexpand("~/foo", &home), "/Users/test/foo");
        assert_eq!(shellexpand("/abs/path", &home), "/abs/path");
    }
}
