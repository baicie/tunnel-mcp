use crate::product::mcp::server::McpServerManager;
use crate::product::settings::{SettingsStore, TunnelSettings};
use crate::product::status::McpServerStatus;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn load_settings(app: &AppHandle) -> Result<TunnelSettings, String> {
    let store = SettingsStore::new(settings_path(app)?);
    store.load().map_err(|err| err.to_string())
}

fn configured_roots(settings: &TunnelSettings) -> Vec<PathBuf> {
    settings
        .resource_root
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![PathBuf::from(value)])
        .unwrap_or_default()
}

#[tauri::command]
pub async fn start_mcp_server(
    app: AppHandle,
    manager: State<'_, McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    let roots = configured_roots(&settings);

    manager
        .start(settings.mcp_server_port, roots)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_mcp_server(
    app: AppHandle,
    manager: State<McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    Ok(manager.stop(settings.mcp_server_port, configured_roots(&settings)))
}

#[tauri::command]
pub fn get_mcp_status(
    app: AppHandle,
    manager: State<McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    Ok(manager.status_with_config(settings.mcp_server_port, configured_roots(&settings)))
}
