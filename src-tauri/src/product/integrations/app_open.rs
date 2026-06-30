#![allow(dead_code)]

use crate::product::mcp::resources::ReadPolicy;
use anyhow::{anyhow, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn open_in_vscode(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<()> {
    let path = authorized_existing_path(path, policy)?;

    let status = Command::new("code").arg(&path).status()?;
    if !status.success() {
        bail!("code CLI exited with status {status}");
    }

    Ok(())
}

pub fn reveal_in_file_manager(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<()> {
    let path = authorized_existing_path(path, policy)?;

    if cfg!(target_os = "macos") {
        Command::new("open").arg("-R").arg(&path).spawn()?;
    } else if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
    } else {
        let target = path.parent().unwrap_or(&path);
        Command::new("xdg-open").arg(target).spawn()?;
    }

    Ok(())
}

fn authorized_existing_path(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<PathBuf> {
    crate::product::security::path_guard::reject_traversal(path)?;

    if !path.exists() {
        return Err(anyhow!("path does not exist: {}", path.display()));
    }

    let canonical = path.canonicalize()?;

    if !policy.can_read(&canonical) {
        bail!("permission denied");
    }

    Ok(canonical)
}
