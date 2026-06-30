use crate::product::mcp::resources::ReadPolicy;
use crate::product::mcp::server::McpServerManager;
use crate::product::permissions::read_policy::PermissionReadPolicy;
use crate::product::permissions::store::PermissionStore;
use crate::product::settings::{SettingsStore, TunnelSettings};
use crate::product::status::McpServerStatus;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn permission_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("permissions.json"))
        .map_err(|err| err.to_string())
}

fn load_settings(app: &AppHandle) -> Result<TunnelSettings, String> {
    let store = SettingsStore::new(settings_path(app)?);
    store.load().map_err(|err| err.to_string())
}

fn build_read_policy(app: &AppHandle) -> Result<Arc<PermissionReadPolicy>, String> {
    let scopes = PermissionStore::new(permission_path(app)?)
        .list()
        .map_err(|err| err.to_string())?;

    PermissionReadPolicy::new(scopes)
        .map(Arc::new)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn start_mcp_server(
    app: AppHandle,
    manager: State<'_, McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    let policy = build_read_policy(&app)?;

    manager
        .start(settings.mcp_server_port, policy)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_mcp_server(
    app: AppHandle,
    manager: State<McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    let policy = build_read_policy(&app)?;

    Ok(manager.stop(settings.mcp_server_port, policy.authorized_roots()))
}

#[tauri::command]
pub fn get_mcp_status(
    app: AppHandle,
    manager: State<McpServerManager>,
) -> Result<McpServerStatus, String> {
    let settings = load_settings(&app)?;
    let policy = build_read_policy(&app)?;

    Ok(manager.status_with_config(settings.mcp_server_port, policy.authorized_roots()))
}
