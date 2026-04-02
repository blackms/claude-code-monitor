use super::models::{HistoryEntry, SessionInfo};
use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn parse_history(path: &Path) -> Result<Vec<HistoryEntry>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<HistoryEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(_) => continue, // Skip malformed lines
        }
    }

    Ok(entries)
}

pub fn sync_history_archive(
    archive_path: &Path,
    recent_entries: &[HistoryEntry],
) -> Result<Vec<HistoryEntry>> {
    let mut all_entries = Vec::new();

    // 1. Load existing archive if it exists
    if archive_path.exists() {
        if let Ok(archived) = parse_history(archive_path) {
            all_entries.extend(archived);
        }
    }

    // 2. Add recent entries, keeping track of seen timestamps+session_ids to deduplicate
    let mut seen: std::collections::HashSet<(u64, String)> = all_entries
        .iter()
        .map(|e| (e.timestamp, e.session_id.clone()))
        .collect();

    let mut new_additions = 0;
    for entry in recent_entries {
        let key = (entry.timestamp, entry.session_id.clone());
        if !seen.contains(&key) {
            seen.insert(key);
            all_entries.push(entry.clone());
            new_additions += 1;
        }
    }

    // 3. Sort by timestamp ascending
    all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    // 4. Save back to archive if we added new entries
    if new_additions > 0 {
        if let Ok(file) = std::fs::File::create(archive_path) {
            let mut writer = std::io::BufWriter::new(file);
            for entry in &all_entries {
                if let Ok(json) = serde_json::to_string(entry) {
                    use std::io::Write;
                    let _ = writeln!(writer, "{}", json);
                }
            }
        }
    }

    Ok(all_entries)
}

pub fn group_sessions(
    entries: &[HistoryEntry],
    current_session_id: Option<&str>,
) -> Vec<SessionInfo> {
    let mut sessions: HashMap<String, SessionInfo> = HashMap::new();

    for entry in entries {
        let project_name = extract_project_name(&entry.project);

        sessions
            .entry(entry.session_id.clone())
            .and_modify(|s| {
                s.last_timestamp = s.last_timestamp.max(entry.timestamp);
                s.first_timestamp = s.first_timestamp.min(entry.timestamp);
                s.message_count += 1;
            })
            .or_insert_with(|| SessionInfo {
                session_id: entry.session_id.clone(),
                project: entry.project.clone(),
                project_name,
                first_timestamp: entry.timestamp,
                last_timestamp: entry.timestamp,
                message_count: 1,
                is_active: current_session_id.map_or(false, |id| id == entry.session_id),
            });
    }

    let mut session_list: Vec<SessionInfo> = sessions.into_values().collect();
    session_list.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

    // Mark the most recent session as potentially active if no current_session_id
    if current_session_id.is_none() {
        if let Some(first) = session_list.first_mut() {
            // Check if last activity was within 5 minutes
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            if now - first.last_timestamp < 5 * 60 * 1000 {
                first.is_active = true;
            }
        }
    }

    session_list
}

fn extract_project_name(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    match name.as_str() {
        "massima-energia" | "massima energia" | "crm-wfa" | "crm-wfa-fe" | "Massima Energia" => {
            "Massima Energia".to_string()
        }
        _ => name,
    }
}

/// Count messages from today based on history entries
pub fn count_today_messages(entries: &[HistoryEntry]) -> u64 {
    let today = chrono::Local::now().date_naive();

    entries
        .iter()
        .filter(|e| {
            let dt = chrono::DateTime::from_timestamp_millis(e.timestamp as i64);
            dt.map(|d| d.with_timezone(&chrono::Local).date_naive() == today)
                .unwrap_or(false)
        })
        .count() as u64
}

/// Count messages from the last N hours
pub fn count_recent_messages(entries: &[HistoryEntry], hours: u64) -> u64 {
    let cutoff = chrono::Utc::now().timestamp_millis() as u64 - (hours * 60 * 60 * 1000);

    entries.iter().filter(|e| e.timestamp >= cutoff).count() as u64
}

/// Project statistics
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct ProjectStats {
    pub name: String,
    pub path: String,
    pub message_count: u64,
    pub estimated_tokens: u64,
    pub estimated_cost: f64,
    pub last_timestamp: u64,
}

/// Count messages per project and return top N, populating proportional estimates
pub fn count_projects(
    entries: &[HistoryEntry],
    top_n: usize,
    total_global_tokens: u64,
    total_global_cost: f64,
) -> Vec<ProjectStats> {
    let total_history_messages = entries.len();
    let mut project_counts: HashMap<String, (String, u64, u64)> = HashMap::new();

    for entry in entries {
        let project_name = extract_project_name(&entry.project);

        // Hide specific projects from the dashboard
        if project_name == "world_data" || project_name == "Current" {
            continue;
        }

        project_counts
            .entry(project_name.clone())
            .and_modify(|(_, count, last_ts)| {
                *count += 1;
                if entry.timestamp > *last_ts {
                    *last_ts = entry.timestamp;
                }
            })
            .or_insert((entry.project.clone(), 1, entry.timestamp));
    }

    let mut projects: Vec<ProjectStats> = project_counts
        .into_iter()
        .map(|(name, (path, count, last_ts))| {
            let ratio = if total_history_messages > 0 {
                count as f64 / total_history_messages as f64
            } else {
                0.0
            };

            ProjectStats {
                name,
                path,
                message_count: count,
                estimated_tokens: (total_global_tokens as f64 * ratio) as u64,
                estimated_cost: total_global_cost * ratio,
                last_timestamp: last_ts,
            }
        })
        .collect();

    projects.sort_by(|a, b| b.message_count.cmp(&a.message_count));
    if top_n > 0 {
        projects.truncate(top_n);
    }
    projects
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_history() {
        let jsonl = r#"{"display":"test","pastedContents":{},"timestamp":1000,"project":"/test/project","sessionId":"abc123"}
{"display":"test2","pastedContents":{},"timestamp":2000,"project":"/test/project","sessionId":"abc123"}"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(jsonl.as_bytes()).unwrap();

        let entries = parse_history(file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].session_id, "abc123");
    }

    #[test]
    fn test_group_sessions() {
        let entries = vec![
            HistoryEntry {
                display: "test".to_string(),
                pasted_contents: HashMap::new(),
                timestamp: 1000,
                project: "/home/user/my-project".to_string(),
                session_id: "session1".to_string(),
            },
            HistoryEntry {
                display: "test2".to_string(),
                pasted_contents: HashMap::new(),
                timestamp: 2000,
                project: "/home/user/my-project".to_string(),
                session_id: "session1".to_string(),
            },
        ];

        let sessions = group_sessions(&entries, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].message_count, 2);
        assert_eq!(sessions[0].project_name, "my-project");
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(extract_project_name("/home/user/my-project"), "my-project");
        assert_eq!(
            extract_project_name("/Users/test/Projects/cool-app"),
            "cool-app"
        );
    }
}
