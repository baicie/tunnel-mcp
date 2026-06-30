use super::event::ListLogsInput;
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
    let path = output_dir.join("tunnel-mcp-diagnostics.json");
    fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
    Ok(path)
}
