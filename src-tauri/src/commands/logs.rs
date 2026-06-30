use crate::product::logs::diagnostics::export_diagnostics as export_diagnostics_file;
use crate::product::logs::event::{AuditLogEvent, ListLogsInput};
use crate::product::logs::store::AuditLogStore;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn logs_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("logs.ndjson"))
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_logs(app: AppHandle, input: ListLogsInput) -> Result<Vec<AuditLogEvent>, String> {
    AuditLogStore::new(logs_path(&app)?)
        .list(input)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn export_diagnostics(app: AppHandle) -> Result<String, String> {
    let app_data = app.path().app_data_dir().map_err(|err| err.to_string())?;
    let output = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| app_data.clone());
    export_diagnostics_file(&app_data, &output)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|err| err.to_string())
}
