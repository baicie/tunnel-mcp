use crate::product::permissions::policy::PermissionPolicy;
use crate::product::permissions::scope::{
    NewPermissionScope, PermissionAccess, PermissionDecision, PermissionScope,
};
use crate::product::permissions::store::PermissionStore;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn permission_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("permissions.json"))
        .map_err(|err| err.to_string())
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
    PermissionStore::new(permission_path(&app)?)
        .add(scope)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn remove_permission_scope(app: AppHandle, id: String) -> Result<Vec<PermissionScope>, String> {
    PermissionStore::new(permission_path(&app)?)
        .remove(&id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn check_permission(
    app: AppHandle,
    path: String,
    access: PermissionAccess,
) -> Result<PermissionDecision, String> {
    let scopes = PermissionStore::new(permission_path(&app)?)
        .list()
        .map_err(|err| err.to_string())?;
    let policy = PermissionPolicy::new(scopes).map_err(|err| err.to_string())?;
    Ok(policy.check_path(PathBuf::from(path).as_path(), access))
}
