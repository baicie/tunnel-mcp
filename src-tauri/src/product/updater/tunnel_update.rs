use crate::product::settings::TunnelSettings;
use crate::product::tunnel::client_binary::{select_asset, TunnelClientManifest};
use anyhow::anyhow;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelClientVersionStatus {
    pub installed: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

/// Fetch the tunnel-client manifest from `manifest_url` and compare its
/// version against the version currently installed in `settings`.
pub async fn check_tunnel_client_update(
    settings: &TunnelSettings,
    manifest_url: &str,
) -> anyhow::Result<TunnelClientVersionStatus> {
    let manifest_text = reqwest::get(manifest_url)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let manifest: TunnelClientManifest = serde_json::from_str(&manifest_text)?;

    select_asset(&manifest)
        .ok_or_else(|| anyhow!("no tunnel-client asset for current platform"))?;

    let current = settings
        .tunnel_client_path
        .as_ref()
        .and_then(|path| version_from_path(path));

    Ok(TunnelClientVersionStatus {
        installed: settings
            .tunnel_client_path
            .as_ref()
            .is_some_and(|value| !value.is_empty()),
        update_available: current.as_deref() != Some(manifest.version.as_str()),
        current_version: current,
        latest_version: Some(manifest.version),
    })
}

/// Derive the version segment from a binary path of the shape
/// `<bin_root>/<version>/tunnel-client[.exe]`. Returns `None` when the
/// path does not match that convention so callers can fall back to
/// "unknown current version".
pub fn version_from_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .map(|value| value.to_string())
}

/// Find the highest version directory under `<app_data_dir>/bin/` that
/// is not `current_version`. We pick the highest lexicographically so
/// that the most recently installed predecessor is selected first,
/// matching the assumption that versions sort ascending.
pub fn previous_version_dir(
    app_data_dir: &Path,
    current_version: &str,
) -> anyhow::Result<Option<PathBuf>> {
    let bin_root = app_data_dir.join("bin");
    if !bin_root.exists() {
        return Ok(None);
    }

    let mut versions = fs::read_dir(bin_root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(|value| value.to_string()))
        .filter(|version| version != current_version)
        .collect::<Vec<_>>();

    versions.sort();
    Ok(versions
        .pop()
        .map(|version| app_data_dir.join("bin").join(version)))
}

/// Switch the configured `tunnel_client_path` to the binary under the
/// highest previously installed version directory. Returns the status
/// snapshot describing the resulting install state.
pub fn rollback_tunnel_client(
    app_data_dir: &Path,
    settings: &mut TunnelSettings,
) -> anyhow::Result<TunnelClientVersionStatus> {
    let current_version = settings
        .tunnel_client_path
        .as_ref()
        .and_then(|value| version_from_path(value));

    let Some(current_version) = current_version else {
        return Ok(TunnelClientVersionStatus {
            installed: false,
            current_version: None,
            latest_version: None,
            update_available: false,
        });
    };

    let Some(previous_dir) = previous_version_dir(app_data_dir, &current_version)? else {
        return Ok(TunnelClientVersionStatus {
            installed: true,
            current_version: Some(current_version),
            latest_version: None,
            update_available: false,
        });
    };

    let binary = if cfg!(windows) {
        previous_dir.join("tunnel-client.exe")
    } else {
        previous_dir.join("tunnel-client")
    };

    let new_path = binary.to_string_lossy().to_string();
    let new_version = version_from_path(&new_path);

    settings.tunnel_client_path = Some(new_path);

    Ok(TunnelClientVersionStatus {
        installed: true,
        current_version: new_version,
        latest_version: None,
        update_available: false,
    })
}
