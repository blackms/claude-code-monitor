use std::path::PathBuf;
use std::time::Duration;

#[allow(dead_code)]
pub struct Config {
    pub claude_dir: PathBuf,
    pub stats_file: PathBuf,
    pub history_file: PathBuf,
    pub refresh_rate: Duration,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let claude_dir = home.join(".claude");

        Ok(Self {
            stats_file: claude_dir.join("stats-cache.json"),
            history_file: claude_dir.join("history.jsonl"),
            claude_dir,
            refresh_rate: Duration::from_millis(100),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Failed to create default config")
    }
}
