use crate::product::approvals::request::ApprovalRequest;
use crate::product::approvals::store::ApprovalStore;
use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn approvals_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("approvals.json"))
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
pub fn list_approval_requests(app: AppHandle) -> Result<Vec<ApprovalRequest>, String> {
    ApprovalStore::new(approvals_path(&app)?)
        .list()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn approve_request(app: AppHandle, id: String) -> Result<ApprovalRequest, String> {
    let audit = audit_store(&app)?;

    match ApprovalStore::new(approvals_path(&app)?).approve(&id) {
        Ok(request) => {
            append_audit_log(
                &audit,
                Some(request.id.clone()),
                "approval.approved",
                LogLevel::Info,
                "approval request approved",
                json!({
                    "approvalId": request.id,
                    "tool": request.tool,
                    "targetPath": request.target_path,
                    "expiresAt": request.expires_at,
                }),
            );
            Ok(request)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                Some(id),
                "approval.rejected",
                LogLevel::Warn,
                "approval approve failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}

#[tauri::command]
pub fn reject_request(app: AppHandle, id: String) -> Result<ApprovalRequest, String> {
    let audit = audit_store(&app)?;

    match ApprovalStore::new(approvals_path(&app)?).reject(&id) {
        Ok(request) => {
            append_audit_log(
                &audit,
                Some(request.id.clone()),
                "approval.rejected",
                LogLevel::Info,
                "approval request rejected",
                json!({
                    "approvalId": request.id,
                    "tool": request.tool,
                    "targetPath": request.target_path,
                    "expiresAt": request.expires_at,
                }),
            );
            Ok(request)
        }
        Err(err) => {
            let message = err.to_string();
            append_audit_log(
                &audit,
                Some(id),
                "approval.rejected",
                LogLevel::Warn,
                "approval reject failed",
                json!({ "error": message }),
            );
            Err(message)
        }
    }
}
