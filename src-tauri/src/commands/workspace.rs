use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::workspace::profile::{SaveWorkspaceProfileInput, WorkspaceProfile};
use crate::product::workspace::store::WorkspaceStore;
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn workspace_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("workspaces.json"))
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

#[tauri::command]
pub fn list_workspace_profiles(app: AppHandle) -> Result<Vec<WorkspaceProfile>, String> {
    WorkspaceStore::new(workspace_path(&app)?)
        .list()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_workspace_profile(
    app: AppHandle,
    profile: SaveWorkspaceProfileInput,
) -> Result<WorkspaceProfile, String> {
    let audit = match audit_store(&app) {
        Ok(store) => Some(store),
        Err(e) => {
            log::warn!("failed to create audit store: {e}");
            None
        }
    };

    match WorkspaceStore::new(workspace_path(&app)?).save_profile(profile) {
        Ok(profile) => {
            if let Some(ref store) = audit {
                append_audit_log(
                    store,
                    None,
                    "workspace.saved",
                    LogLevel::Info,
                    "workspace profile saved",
                    json!({
                        "workspaceId": profile.id,
                        "name": profile.name,
                        "rootPath": profile.root_path,
                        "permissionScopeCount": profile.permission_scopes.len(),
                    }),
                );
            }
            Ok(profile)
        }
        Err(err) => {
            let message = err.to_string();
            if let Some(ref store) = audit {
                append_audit_log(
                    store,
                    None,
                    "workspace.error",
                    LogLevel::Error,
                    "workspace profile save failed",
                    json!({ "error": message }),
                );
            }
            Err(message)
        }
    }
}

#[tauri::command]
pub fn remove_workspace_profile(
    app: AppHandle,
    id: String,
) -> Result<Vec<WorkspaceProfile>, String> {
    let audit = match audit_store(&app) {
        Ok(store) => Some(store),
        Err(e) => {
            log::warn!("failed to create audit store: {e}");
            None
        }
    };

    match WorkspaceStore::new(workspace_path(&app)?).remove(&id) {
        Ok(profiles) => {
            if let Some(ref store) = audit {
                append_audit_log(
                    store,
                    None,
                    "workspace.removed",
                    LogLevel::Info,
                    "workspace profile removed",
                    json!({ "workspaceId": id }),
                );
            }
            Ok(profiles)
        }
        Err(err) => {
            let message = err.to_string();
            if let Some(ref store) = audit {
                append_audit_log(
                    store,
                    None,
                    "workspace.error",
                    LogLevel::Error,
                    "workspace profile remove failed",
                    json!({ "workspaceId": id, "error": message }),
                );
            }
            Err(message)
        }
    }
}
