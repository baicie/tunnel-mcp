use crate::product::settings::{
    to_public_settings, PublicTunnelSettings, SettingsError, SettingsStore, TunnelSettings,
};
use crate::product::status::{initial_mcp_status, McpServerStatus, TunnelStatus};
use crate::product::tunnel::client_process::TunnelProcessManager;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn map_error(error: SettingsError) -> String {
    error.to_string()
}

#[tauri::command]
pub fn get_tunnel_settings(app: AppHandle) -> Result<PublicTunnelSettings, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    store.load().map(to_public_settings).map_err(map_error)
}

#[tauri::command]
pub fn save_tunnel_settings(
    app: AppHandle,
    settings: TunnelSettings,
) -> Result<PublicTunnelSettings, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    store
        .save(settings)
        .map(to_public_settings)
        .map_err(map_error)
}

#[tauri::command]
pub fn get_tunnel_status(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    let settings = store.load().map_err(map_error)?;
    manager.status(&settings).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_mcp_status(app: AppHandle) -> Result<McpServerStatus, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    let settings = store.load().map_err(map_error)?;
    Ok(initial_mcp_status(settings.mcp_server_port))
}
