use crate::product::settings::SettingsStore;
use crate::product::status::{initial_tunnel_status, TunnelStatus};
use crate::product::tunnel::client_download::{
    install_tunnel_client as install_binary, InstallTunnelClientInput,
};
use crate::product::tunnel::client_health::DEFAULT_LOCAL_MCP_URL;
use crate::product::tunnel::client_process::TunnelProcessManager;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn load_settings(app: &AppHandle) -> Result<crate::product::settings::TunnelSettings, String> {
    let store = SettingsStore::new(settings_path(app)?);
    store.load().map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn install_tunnel_client(
    app: AppHandle,
    input: InstallTunnelClientInput,
) -> Result<TunnelStatus, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    let installed = install_binary(&app_data_dir, input)
        .await
        .map_err(|err| err.to_string())?;

    let mut settings = load_settings(&app)?;
    settings.tunnel_client_path = Some(installed.path);

    let store = SettingsStore::new(settings_path(&app)?);
    let saved = store.save(settings).map_err(|err| err.to_string())?;

    Ok(initial_tunnel_status(saved.tunnel_client_path))
}

#[tauri::command]
pub fn start_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let settings = load_settings(&app)?;
    manager
        .start(&settings, DEFAULT_LOCAL_MCP_URL)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let settings = load_settings(&app)?;
    manager.stop(&settings).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn restart_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let settings = load_settings(&app)?;
    manager
        .restart(&settings, DEFAULT_LOCAL_MCP_URL)
        .map_err(|err| err.to_string())
}
