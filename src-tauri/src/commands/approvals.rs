use crate::product::approvals::request::ApprovalRequest;
use crate::product::approvals::store::ApprovalStore;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn approvals_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("approvals.json"))
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_approval_requests(app: AppHandle) -> Result<Vec<ApprovalRequest>, String> {
    ApprovalStore::new(approvals_path(&app)?)
        .list()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn approve_request(app: AppHandle, id: String) -> Result<ApprovalRequest, String> {
    ApprovalStore::new(approvals_path(&app)?)
        .approve(&id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn reject_request(app: AppHandle, id: String) -> Result<ApprovalRequest, String> {
    ApprovalStore::new(approvals_path(&app)?)
        .reject(&id)
        .map_err(|err| err.to_string())
}
