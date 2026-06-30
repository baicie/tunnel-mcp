use super::store::ApprovalStore;
use crate::product::permissions::policy::PermissionPolicy;
use crate::product::permissions::scope::PermissionAccess;
use anyhow::{anyhow, bail};
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn write_file_with_approval(
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    approval_id: &str,
    target_path: PathBuf,
    content: &str,
) -> anyhow::Result<()> {
    let decision = policy.check_path(&target_path, PermissionAccess::Write);
    if !decision.allowed {
        bail!("permission denied: {}", decision.reason);
    }

    let approval = approval_store.get_valid_approved(approval_id)?;
    if approval.target_path != target_path.to_string_lossy() {
        return Err(anyhow!("approval target path mismatch"));
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target_path, content)?;
    Ok(())
}
