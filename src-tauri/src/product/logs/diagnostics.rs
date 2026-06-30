use super::event::ListLogsInput;
use super::redact::redact_value;
use super::store::AuditLogStore;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub fn export_diagnostics(app_data_dir: &Path, output_dir: &Path) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(output_dir)?;

    let log_store = AuditLogStore::new(app_data_dir.join("logs.ndjson"));

    let logs = log_store.list(ListLogsInput {
        r#type: None,
        request_id: None,
        limit: Some(500),
    })?;

    let payload = json!({
        "version": 1,
        "generatedAt": chrono::Utc::now().to_rfc3339(),
        "logs": logs,
    });

    // Re-redact the entire payload at export time so any historical
    // entries written before redaction existed — or by future writers
    // that bypass `AuditLogStore::append` — are still scrubbed.
    let redacted_payload = redact_value(payload);

    let path = output_dir.join("tunnel-mcp-diagnostics.json");

    fs::write(&path, serde_json::to_string_pretty(&redacted_payload)?)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::export_diagnostics;
    use crate::product::logs::event::LogLevel;
    use crate::product::logs::store::AuditLogStore;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn diagnostics_export_is_redacted() {
        let app_data = tempdir().unwrap();
        let output = tempdir().unwrap();

        let store = AuditLogStore::new(app_data.path().join("logs.ndjson"));
        store
            .append(
                Some("req1".to_string()),
                "tunnel.start",
                LogLevel::Info,
                "openai_api_key=sk-1234567890abcdef",
                json!({
                    "token": "abc",
                    "nested": {
                        "secret": "def"
                    }
                }),
            )
            .unwrap();

        let path = export_diagnostics(app_data.path(), output.path()).unwrap();
        let raw = fs::read_to_string(path).unwrap();

        assert!(!raw.contains("sk-1234567890abcdef"));
        assert!(!raw.contains("\"abc\""));
        assert!(!raw.contains("\"def\""));
        assert!(raw.contains("[REDACTED]"));
    }
}
