use super::request::ApprovalTool;
use super::store::ApprovalStore;
use super::write_log::{WriteLogEntry, WriteLogStatus, WriteLogStore};
use crate::product::permissions::policy::PermissionPolicy;
use anyhow::{anyhow, bail, Context};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn canonical_write_target(path: &Path) -> anyhow::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize().context("canonicalize target file");
    }

    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("target path has no parent"))?
        .canonicalize()
        .context("canonicalize target parent")?;

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("target path has no file name"))?;

    Ok(parent.join(file_name))
}

pub fn write_file_with_approval(
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    write_log_store: &WriteLogStore,
    approval_id: &str,
    target_path: PathBuf,
    content: &str,
) -> anyhow::Result<()> {
    let canonical_target = canonical_write_target(&target_path)?;
    let canonical_target_text = canonical_target.to_string_lossy().to_string();
    let content_sha256 = sha256_hex(content);

    let result = write_file_inner(
        policy,
        approval_store,
        approval_id,
        canonical_target.clone(),
        canonical_target_text.clone(),
        content,
        content_sha256.clone(),
    );

    let status = if result.is_ok() {
        WriteLogStatus::Succeeded
    } else {
        WriteLogStatus::Failed
    };

    let error = result.as_ref().err().map(|err| err.to_string());

    let _ = write_log_store.append(WriteLogEntry {
        id: String::new(),
        approval_id: approval_id.to_string(),
        tool: ApprovalTool::FilesWrite,
        target_path: canonical_target_text,
        content_sha256: Some(content_sha256),
        status,
        error,
        created_at: 0,
    });

    result
}

fn write_file_inner(
    policy: &PermissionPolicy,
    approval_store: &ApprovalStore,
    approval_id: &str,
    canonical_target: PathBuf,
    canonical_target_text: String,
    content: &str,
    content_sha256: String,
) -> anyhow::Result<()> {
    let decision = policy.check_write_target(&canonical_target);
    if !decision.allowed {
        bail!("permission denied: {}", decision.reason);
    }

    let approval = approval_store.get_valid_approved(
        approval_id,
        ApprovalTool::FilesWrite,
        &canonical_target_text,
        Some(&content_sha256),
    )?;

    if approval.target_path != canonical_target_text {
        bail!("approval target path mismatch");
    }

    if let Some(parent) = canonical_target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(canonical_target, content)?;
    Ok(())
}
