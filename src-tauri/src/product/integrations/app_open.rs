#![allow(dead_code)]

use anyhow::{anyhow, bail};
use std::path::Path;
use std::process::Command;

/// Open a file or directory in VS Code via the `code` CLI.
/// Caller is responsible for verifying the path is authorized.
pub fn open_in_vscode(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow!("path does not exist: {}", path.display()));
    }
    let status = Command::new("code").arg(path).status()?;
    if !status.success() {
        bail!("code CLI exited with status {status}");
    }
    Ok(())
}

/// Reveal a file or directory in the OS file manager.
/// Always uses passive invocations (`open -R`, `explorer /select,`, `xdg-open`);
/// this never writes or modifies user files.
pub fn reveal_in_file_manager(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow!("path does not exist: {}", path.display()));
    }

    if cfg!(target_os = "macos") {
        Command::new("open").arg("-R").arg(path).spawn()?;
    } else if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
    } else {
        let target = path.parent().unwrap_or(path);
        Command::new("xdg-open").arg(target).spawn()?;
    }
    Ok(())
}
