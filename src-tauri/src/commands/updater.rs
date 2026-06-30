use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::settings::SettingsStore;
use crate::product::updater::app_update::{no_update, UpdateCheckResult};
use crate::product::updater::tunnel_update::{
    check_tunnel_client_update as check_client_update, rollback_tunnel_client as rollback_client,
    TunnelClientVersionStatus,
};
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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

/// Phase 8 placeholder: returns a "no update available" result while
/// the Tauri updater is wired with signing keys and a `latest.json`
/// release pipeline. Uses `package_info()` so the reported version
/// always matches what Tauri actually bundles.
#[tauri::command]
pub fn check_app_update(app: AppHandle) -> UpdateCheckResult {
    no_update(app.package_info().version.to_string())
}

#[tauri::command]
pub async fn check_tunnel_client_update(
    app: AppHandle,
    manifest_url: String,
) -> Result<TunnelClientVersionStatus, String> {
    let audit = audit_store(&app)?;
    let store = SettingsStore::new(settings_path(&app)?);
    let settings = store.load().map_err(|err| err.to_string())?;

    append_audit_log(
        &audit,
        None,
        "tunnel.update",
        LogLevel::Info,
        "check tunnel-client update requested",
        json!({ "manifestUrl": manifest_url }),
    );

    match check_client_update(&settings, &manifest_url).await {
        Ok(status) => {
            append_audit_log(
                &audit,
                None,
                "tunnel.update",
                LogLevel::Info,
                "tunnel-client update check completed",
                json!({
                    "installed": status.installed,
                    "currentVersion": status.current_version,
                    "latestVersion": status.latest_version,
                    "updateAvailable": status.update_available,
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
                "tunnel-client update check failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn rollback_tunnel_client(app: AppHandle) -> Result<TunnelClientVersionStatus, String> {
    let audit = audit_store(&app)?;
    let app_data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    let store = SettingsStore::new(settings_path(&app)?);
    let mut settings = store.load().map_err(|err| err.to_string())?;

    append_audit_log(
        &audit,
        None,
        "tunnel.update",
        LogLevel::Info,
        "rollback tunnel-client requested",
        json!({}),
    );

    let status = rollback_client(&app_data_dir, &mut settings).map_err(|err| err.to_string())?;
    store.save(settings).map_err(|err| err.to_string())?;

    append_audit_log(
        &audit,
        None,
        "tunnel.update",
        LogLevel::Info,
        "rollback tunnel-client completed",
        json!({
            "installed": status.installed,
            "currentVersion": status.current_version,
        }),
    );

    Ok(status)
}
