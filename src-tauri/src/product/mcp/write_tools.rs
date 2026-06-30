use crate::product::approvals::request::{ApprovalTool, NewApprovalRequest};
use crate::product::approvals::store::ApprovalStore;
use crate::product::approvals::write_guard::{
    canonical_write_target, sha256_hex, write_file_with_approval,
};
use crate::product::approvals::write_log::WriteLogStore;
use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
use crate::product::permissions::policy::PermissionPolicy;
use anyhow::{anyhow, bail};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const DEFAULT_WRITE_APPROVAL_TTL_SECONDS: i64 = 300;

pub enum WriteToolResult {
    ApprovalRequired(Value),
    Written(Value),
}

pub fn handle_files_write(
    request_id: Option<String>,
    params: &Value,
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    write_log_store: &WriteLogStore,
    audit_log_store: &AuditLogStore,
) -> anyhow::Result<WriteToolResult> {
    let target_path = params
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("params.path is required"))?;

    let content = params
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("params.content is required"))?;

    let canonical_target = canonical_write_target(&target_path)?;
    let canonical_target_text = canonical_target.to_string_lossy().to_string();
    let content_sha256 = sha256_hex(content);

    let decision = policy.check_write_target(&canonical_target);
    if !decision.allowed {
        append_audit_log(
            audit_log_store,
            request_id.clone(),
            "permission.deny",
            LogLevel::Warn,
            "write permission denied",
            json!({
                "path": canonical_target_text,
                "reason": decision.reason,
                "requireApproval": decision.require_approval,
            }),
        );

        bail!("permission denied: {}", decision.reason);
    }

    append_audit_log(
        audit_log_store,
        request_id.clone(),
        "permission.allow",
        LogLevel::Info,
        "write permission allowed",
        json!({
            "path": canonical_target_text,
            "requireApproval": decision.require_approval,
        }),
    );

    if let Some(approval_id) = params.get("approvalId").and_then(Value::as_str) {
        write_file_with_approval(
            policy,
            approval_store,
            write_log_store,
            approval_id,
            canonical_target,
            content,
        )?;

        append_audit_log(
            audit_log_store,
            request_id,
            "file.write",
            LogLevel::Info,
            "file written through MCP",
            json!({
                "approvalId": approval_id,
                "path": canonical_target_text,
                "bytes": content.len(),
                "contentSha256": content_sha256,
            }),
        );

        return Ok(WriteToolResult::Written(json!({
            "ok": true,
            "path": canonical_target_text,
            "bytes": content.len(),
            "contentSha256": content_sha256,
        })));
    }

    let request = approval_store.create(NewApprovalRequest {
        tool: ApprovalTool::FilesWrite,
        target_path: canonical_target_text.clone(),
        summary: format!("Write {} bytes to {}", content.len(), canonical_target_text),
        diff: params
            .get("diff")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        content_sha256: Some(content_sha256.clone()),
        ttl_seconds: DEFAULT_WRITE_APPROVAL_TTL_SECONDS,
    })?;

    append_audit_log(
        audit_log_store,
        request_id,
        "approval.created",
        LogLevel::Info,
        "approval request created",
        json!({
            "approvalId": request.id,
            "tool": request.tool,
            "targetPath": request.target_path,
            "contentSha256": content_sha256,
            "expiresAt": request.expires_at,
        }),
    );

    Ok(WriteToolResult::ApprovalRequired(json!({
        "approvalRequired": true,
        "approvalId": request.id,
        "status": request.status,
        "targetPath": request.target_path,
        "contentSha256": content_sha256,
        "expiresAt": request.expires_at,
    })))
}
