use super::event::{AuditLogEvent, ListLogsInput, LogLevel};
use super::redact::{redact_text, redact_value};
use chrono::Utc;
use serde_json::Value;
use std::cmp::Reverse;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

pub struct AuditLogStore {
    path: PathBuf,
}

impl AuditLogStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(
        &self,
        request_id: Option<String>,
        r#type: impl Into<String>,
        level: LogLevel,
        message: impl Into<String>,
        metadata: Value,
    ) -> anyhow::Result<AuditLogEvent> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let event = AuditLogEvent {
            id: Uuid::new_v4().to_string(),
            request_id,
            r#type: r#type.into(),
            level,
            message: redact_text(&message.into()),
            metadata: redact_value(metadata),
            created_at: Utc::now().timestamp_millis(),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, "{}", serde_json::to_string(&event)?)?;
        Ok(event)
    }

    pub fn list(&self, input: ListLogsInput) -> anyhow::Result<Vec<AuditLogEvent>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let file = fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };

            if line.trim().is_empty() {
                continue;
            }

            let Ok(event) = serde_json::from_str::<AuditLogEvent>(&line) else {
                // Skip malformed lines so one corrupt entry cannot break
                // the Logs page or the diagnostics export.
                continue;
            };

            if input
                .r#type
                .as_ref()
                .is_some_and(|value| value != &event.r#type)
            {
                continue;
            }

            if input
                .request_id
                .as_ref()
                .is_some_and(|value| event.request_id.as_ref() != Some(value))
            {
                continue;
            }

            events.push(event);
        }

        events.sort_by_key(|event| Reverse(event.created_at));

        let limit = input.limit.unwrap_or(200).min(1000);
        events.truncate(limit);

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::super::event::{ListLogsInput, LogLevel};
    use super::AuditLogStore;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn append_and_filter_logs() {
        let dir = tempdir().unwrap();
        let store = AuditLogStore::new(dir.path().join("logs.ndjson"));
        store
            .append(
                Some("req1".to_string()),
                "mcp.request",
                LogLevel::Info,
                "read file",
                json!({ "path": "/tmp/a" }),
            )
            .unwrap();
        store
            .append(
                Some("req2".to_string()),
                "permission.deny",
                LogLevel::Warn,
                "deny",
                json!({}),
            )
            .unwrap();

        let logs = store
            .list(ListLogsInput {
                r#type: Some("mcp.request".to_string()),
                request_id: None,
                limit: Some(10),
            })
            .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].r#type, "mcp.request");
    }

    #[test]
    fn append_redacts_metadata() {
        let dir = tempdir().unwrap();
        let store = AuditLogStore::new(dir.path().join("logs.ndjson"));
        let event = store
            .append(
                None,
                "tunnel.start",
                LogLevel::Info,
                "start",
                json!({ "openaiKey": "sk-secret-value" }),
            )
            .unwrap();
        assert_eq!(event.metadata["openaiKey"], "[REDACTED]");
    }

    #[test]
    fn list_skips_malformed_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("logs.ndjson");

        std::fs::write(
            &path,
            "not-json\n{\"id\":\"1\",\"type\":\"mcp.request\",\"level\":\"info\",\"message\":\"ok\",\"metadata\":{},\"createdAt\":1}\n",
        )
        .unwrap();

        let store = AuditLogStore::new(path);
        let logs = store
            .list(ListLogsInput {
                r#type: None,
                request_id: None,
                limit: Some(10),
            })
            .unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].r#type, "mcp.request");
    }

    #[test]
    fn list_clamps_limit_to_max() {
        let dir = tempdir().unwrap();
        let store = AuditLogStore::new(dir.path().join("logs.ndjson"));

        for index in 0..5 {
            store
                .append(
                    None,
                    "mcp.request",
                    LogLevel::Info,
                    format!("req {index}"),
                    json!({ "index": index }),
                )
                .unwrap();
        }

        let logs = store
            .list(ListLogsInput {
                r#type: None,
                request_id: None,
                limit: Some(10_000),
            })
            .unwrap();

        assert_eq!(logs.len(), 5);
    }
}
