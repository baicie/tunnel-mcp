use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::settings::SettingsStore;
use crate::product::status::TunnelStatus;
use crate::product::tunnel::client_download::{
    install_tunnel_client as install_binary, InstallTunnelClientInput,
};
use crate::product::tunnel::client_process::{TunnelClientLogLine, TunnelProcessManager};
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn logs_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("logs.ndjson"))
        .map_err(|err| err.to_string())
}

fn audit_store(app: &AppHandle) -> Result<AuditLogStore, String> {
    Ok(AuditLogStore::new(logs_path(app)?))
}

fn load_settings(app: &AppHandle) -> Result<crate::product::settings::TunnelSettings, String> {
    let store = SettingsStore::new(settings_path(app)?);
    store.load().map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn install_tunnel_client(
    app: AppHandle,
    input: InstallTunnelClientInput,
    manager: State<'_, TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let audit = audit_store(&app)?;

    append_audit_log(
        &audit,
        None,
        "tunnel.download",
        LogLevel::Info,
        "install tunnel-client requested",
        json!({ "version": input.version }),
    );

    let app_data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;

    let installed = match install_binary(&app_data_dir, input).await {
        Ok(value) => value,
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "tunnel.error",
                LogLevel::Error,
                "tunnel-client download failed",
                json!({ "error": message }),
            );
            return Err(message);
        }
    };

    let mut settings = load_settings(&app)?;
    settings.tunnel_client_path = Some(installed.path);
    settings.tunnel_client_version = Some(installed.version);

    let store = SettingsStore::new(settings_path(&app)?);
    let saved = store.save(settings).map_err(|err| err.to_string())?;

    append_audit_log(
        &audit,
        None,
        "tunnel.download",
        LogLevel::Info,
        "tunnel-client installed",
        json!({
            "version": saved.tunnel_client_version,
            "path": saved.tunnel_client_path,
        }),
    );

    manager.status(&saved).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn start_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let audit = audit_store(&app)?;
    let settings = load_settings(&app)?;

    append_audit_log(
        &audit,
        None,
        "tunnel.start",
        LogLevel::Info,
        "start tunnel-client requested",
        json!({
            "hasTunnelId": settings.tunnel_id.is_some(),
            "hasOpenaiApiKey": settings.openai_api_key.is_some(),
            "mcpServerPort": settings.mcp_server_port,
        }),
    );

    match manager.start(&settings) {
        Ok(status) => {
            append_audit_log(
                &audit,
                None,
                "tunnel.start",
                LogLevel::Info,
                "tunnel-client started",
                json!({
                    "running": status.running,
                    "pid": status.pid,
                    "endpoint": status.endpoint,
                    "health": status.health,
                }),
            );
            Ok(status)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "tunnel.error",
                LogLevel::Error,
                "tunnel-client start failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn stop_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let audit = audit_store(&app)?;
    let settings = load_settings(&app)?;

    append_audit_log(
        &audit,
        None,
        "tunnel.stop",
        LogLevel::Info,
        "stop tunnel-client requested",
        json!({}),
    );

    match manager.stop(&settings) {
        Ok(status) => {
            append_audit_log(
                &audit,
                None,
                "tunnel.stop",
                LogLevel::Info,
                "tunnel-client stopped",
                json!({ "running": status.running }),
            );
            Ok(status)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "tunnel.error",
                LogLevel::Error,
                "tunnel-client stop failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn restart_tunnel_client(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let audit = audit_store(&app)?;
    let settings = load_settings(&app)?;

    append_audit_log(
        &audit,
        None,
        "tunnel.stop",
        LogLevel::Info,
        "restart tunnel-client requested",
        json!({}),
    );

    match manager.restart(&settings) {
        Ok(status) => {
            append_audit_log(
                &audit,
                None,
                "tunnel.start",
                LogLevel::Info,
                "tunnel-client restarted",
                json!({
                    "running": status.running,
                    "pid": status.pid,
                    "endpoint": status.endpoint,
                    "health": status.health,
                }),
            );
            Ok(status)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "tunnel.error",
                LogLevel::Error,
                "tunnel-client restart failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn get_tunnel_client_logs(manager: State<TunnelProcessManager>) -> Vec<TunnelClientLogLine> {
    manager.logs()
}
