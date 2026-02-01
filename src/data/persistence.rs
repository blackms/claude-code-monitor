use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::usage_tracker::{QuotaSample, UsageTracker};

/// Serializable sample for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentSample {
    timestamp: DateTime<Utc>,
    session: Option<f64>,
    week: Option<f64>,
}

impl From<&QuotaSample> for PersistentSample {
    fn from(sample: &QuotaSample) -> Self {
        Self {
            timestamp: sample.timestamp,
            session: sample.session_utilization,
            week: sample.week_utilization,
        }
    }
}

impl From<PersistentSample> for QuotaSample {
    fn from(ps: PersistentSample) -> Self {
        Self {
            timestamp: ps.timestamp,
            session_utilization: ps.session,
            week_utilization: ps.week,
        }
    }
}

/// Container for persisted samples
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SamplesFile {
    samples: Vec<PersistentSample>,
    last_updated: DateTime<Utc>,
}

impl UsageTracker {
    /// Save samples to a JSON file
    pub fn save_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let samples: Vec<PersistentSample> = self
            .get_samples()
            .iter()
            .map(PersistentSample::from)
            .collect();

        let file_data = SamplesFile {
            samples,
            last_updated: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&file_data)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, json)?;
        Ok(())
    }

    /// Load samples from a JSON file
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let file_data: SamplesFile = serde_json::from_str(&content)?;

        let mut tracker = Self::default();

        // Filter out samples older than 10 minutes to keep data fresh
        let cutoff = Utc::now() - chrono::Duration::minutes(10);

        for ps in file_data.samples {
            if ps.timestamp > cutoff {
                tracker.add_sample_with_timestamp(ps.timestamp, ps.session, ps.week);
            }
        }

        Ok(tracker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("samples.json");

        let mut tracker = UsageTracker::default();
        tracker.add_sample(Some(50.0), Some(25.0));
        tracker.add_sample(Some(51.0), Some(25.5));

        tracker.save_to_file(&path).unwrap();

        let loaded = UsageTracker::load_from_file(&path).unwrap();
        assert_eq!(loaded.sample_count(), 2);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = UsageTracker::load_from_file(Path::new("/nonexistent/path.json"));
        assert!(result.is_err());
    }
}
