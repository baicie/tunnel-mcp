use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::permissions::policy::PermissionPolicy;
use crate::product::permissions::scope::{
    NewPermissionScope, PermissionAccess, PermissionDecision, PermissionScope,
};
use crate::product::permissions::store::PermissionStore;
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn permission_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("permissions.json"))
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
pub fn list_permission_scopes(app: AppHandle) -> Result<Vec<PermissionScope>, String> {
    PermissionStore::new(permission_path(&app)?)
        .list()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn add_permission_scope(
    app: AppHandle,
    scope: NewPermissionScope,
) -> Result<PermissionScope, String> {
    let audit = audit_store(&app)?;

    match PermissionStore::new(permission_path(&app)?).add(scope) {
        Ok(scope) => {
            append_audit_log(
                &audit,
                None,
                "permission.allow",
                LogLevel::Info,
                "permission scope added",
                json!({
                    "scopeId": scope.id,
                    "kind": scope.kind,
                    "pattern": scope.pattern,
                    "access": scope.access,
                    "requireApproval": scope.require_approval,
                }),
            );
            Ok(scope)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                None,
                "permission.deny",
                LogLevel::Warn,
                "permission scope add failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn remove_permission_scope(app: AppHandle, id: String) -> Result<Vec<PermissionScope>, String> {
    let audit = audit_store(&app)?;
    let result = PermissionStore::new(permission_path(&app)?)
        .remove(&id)
        .map_err(|err| err.to_string())?;

    append_audit_log(
        &audit,
        None,
        "permission.allow",
        LogLevel::Info,
        "permission scope removed",
        json!({ "scopeId": id }),
    );

    Ok(result)
}

#[tauri::command]
pub fn check_permission(
    app: AppHandle,
    path: String,
    access: PermissionAccess,
) -> Result<PermissionDecision, String> {
    let audit = audit_store(&app)?;
    let scopes = PermissionStore::new(permission_path(&app)?)
        .list()
        .map_err(|err| err.to_string())?;

    let policy = PermissionPolicy::new(scopes).map_err(|err| err.to_string())?;
    let decision = policy.check_path(PathBuf::from(&path).as_path(), access.clone());

    append_audit_log(
        &audit,
        None,
        if decision.allowed {
            "permission.allow"
        } else {
            "permission.deny"
        },
        if decision.allowed {
            LogLevel::Info
        } else {
            LogLevel::Warn
        },
        if decision.allowed {
            "permission check allowed"
        } else {
            "permission check denied"
        },
        json!({
            "path": path,
            "access": access,
            "requireApproval": decision.require_approval,
            "reason": decision.reason,
        }),
    );

    Ok(decision)
}
