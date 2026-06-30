use crate::product::approvals::request::{ApprovalTool, NewApprovalRequest};
use crate::product::approvals::store::ApprovalStore;
use crate::product::approvals::write_guard::{
    canonical_write_target, sha256_hex, write_file_with_approval,
};
use crate::product::approvals::write_log::WriteLogStore;
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
    params: &Value,
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    write_log_store: &WriteLogStore,
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
        bail!("permission denied: {}", decision.reason);
    }

    if let Some(approval_id) = params.get("approvalId").and_then(Value::as_str) {
        write_file_with_approval(
            policy,
            approval_store,
            write_log_store,
            approval_id,
            canonical_target,
            content,
        )?;

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

    Ok(WriteToolResult::ApprovalRequired(json!({
        "approvalRequired": true,
        "approvalId": request.id,
        "status": request.status,
        "targetPath": request.target_path,
        "contentSha256": content_sha256,
        "expiresAt": request.expires_at,
    })))
}
