use crate::product::approvals::store::ApprovalStore;
use crate::product::approvals::write_log::WriteLogStore;
use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::mcp::resources::ReadPolicy;
use crate::product::mcp::server::McpServerManager;
use crate::product::mcp::tools::McpWriteContext;
use crate::product::permissions::policy::PermissionPolicy;
use crate::product::permissions::read_policy::PermissionReadPolicy;
use crate::product::permissions::store::PermissionStore;
use crate::product::settings::{SettingsStore, TunnelSettings};
use crate::product::status::McpServerStatus;
use serde_json::json;
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

fn approvals_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("approvals.json"))
        .map_err(|err| err.to_string())
}

fn write_log_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("write-log.json"))
        .map_err(|err| err.to_string())
}

fn local_token_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("local-token.json"))
        .map_err(|err| err.to_string())
}

fn logs_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("logs.ndjson"))
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

fn build_write_context(app: &AppHandle) -> Result<Arc<McpWriteContext>, String> {
    let scopes = PermissionStore::new(permission_path(app)?)
        .list()
        .map_err(|err| err.to_string())?;

    let permission_policy = PermissionPolicy::new(scopes).map_err(|err| err.to_string())?;

    Ok(Arc::new(McpWriteContext {
        permission_policy,
        approval_store: ApprovalStore::new(approvals_path(app)?),
        write_log_store: WriteLogStore::new(write_log_path(app)?),
        audit_log_store: AuditLogStore::new(logs_path(app)?),
    }))
}

#[tauri::command]
pub async fn start_mcp_server(
    app: AppHandle,
    manager: State<'_, McpServerManager>,
) -> Result<McpServerStatus, String> {
    let audit = AuditLogStore::new(logs_path(&app)?);
    let settings = load_settings(&app)?;
    let policy = build_read_policy(&app)?;
    let write_context = build_write_context(&app)?;

    match manager
        .start(settings.mcp_server_port, policy, write_context, local_token_path(&app)?)
        .await
    {
        Ok(status) => {
            append_audit_log(
                &audit,
                None,
                "mcp.response",
                LogLevel::Info,
                "MCP server started",
                json!({
                    "port": status.port,
                    "running": status.running,
                    "authorizedRoots": status.authorized_roots,
                }),
            );
            Ok(status)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "mcp.error",
                LogLevel::Error,
                "MCP server start failed",
                json!({
                    "port": settings.mcp_server_port,
                    "error": message,
                }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn stop_mcp_server(
    app: AppHandle,
    manager: State<McpServerManager>,
) -> Result<McpServerStatus, String> {
    let audit = AuditLogStore::new(logs_path(&app)?);
    let settings = load_settings(&app)?;
    let policy = build_read_policy(&app)?;

    let status = manager.stop(settings.mcp_server_port, policy.authorized_roots());

    append_audit_log(
        &audit,
        None,
        "mcp.response",
        LogLevel::Info,
        "MCP server stopped",
        json!({
            "port": status.port,
            "running": status.running,
        }),
    );

    Ok(status)
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
