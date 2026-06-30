use crate::product::approvals::request::{ApprovalTool, NewApprovalRequest};
use crate::product::approvals::store::ApprovalStore;
use crate::product::permissions::policy::PermissionPolicy;
use crate::product::permissions::scope::PermissionAccess;
use anyhow::bail;
use std::path::PathBuf;

#[allow(dead_code)]
pub const DEFAULT_WRITE_APPROVAL_TTL_SECONDS: i64 = 300;

/// Performs the permission check, then creates a pending approval request
/// for an MCP `files.write` call. Returns the new approval id; the actual
/// write must go through
/// `crate::product::approvals::write_guard::write_file_with_approval`
/// after the user approves.
#[allow(dead_code)]
pub fn create_write_approval(
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    target_path: PathBuf,
    content: &str,
) -> anyhow::Result<String> {
    let decision = policy.check_path(&target_path, PermissionAccess::Write);
    if !decision.allowed {
        bail!("permission denied: {}", decision.reason);
    }

    let summary = format!("Write {} bytes to {}", content.len(), target_path.display());
    let request = approval_store.create(NewApprovalRequest {
        tool: ApprovalTool::FilesWrite,
        target_path: target_path.to_string_lossy().to_string(),
        summary,
        diff: None,
        ttl_seconds: DEFAULT_WRITE_APPROVAL_TTL_SECONDS,
    })?;
    Ok(request.id)
}
