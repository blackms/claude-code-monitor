use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use super::models::{HistoryEntry, SessionInfo};

pub fn parse_history(path: &Path) -> Result<Vec<HistoryEntry>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<HistoryEntry>(line) {
            Ok(entry) => entries.push(entry),
            Err(_) => continue, // Skip malformed lines
        }
    }

    Ok(entries)
}

pub fn group_sessions(entries: &[HistoryEntry], current_session_id: Option<&str>) -> Vec<SessionInfo> {
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
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
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
        assert_eq!(extract_project_name("/Users/test/Projects/cool-app"), "cool-app");
    }
}
