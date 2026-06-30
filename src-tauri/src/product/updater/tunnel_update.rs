use crate::product::settings::TunnelSettings;
use crate::product::tunnel::client_binary::{binary_file_name, select_asset, TunnelClientManifest};
use anyhow::{anyhow, bail};
use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelClientVersionStatus {
    pub installed: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub asset_url: Option<String>,
    pub asset_sha256: Option<String>,
    pub checksum_verified: bool,
}

/// Fetch the tunnel-client manifest from `manifest_url`, pick the
/// platform-specific asset, and compare its version against the
/// currently installed binary in `settings`.
///
/// The manifest asset must declare a non-empty `sha256`. The returned
/// status carries `asset_url` and `asset_sha256` so callers can drive
/// a real download that verifies the checksum before replacing the
/// current binary.
pub async fn check_tunnel_client_update(
    settings: &TunnelSettings,
    manifest_url: &str,
) -> anyhow::Result<TunnelClientVersionStatus> {
    let manifest_text = http_client()?
        .get(manifest_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let manifest: TunnelClientManifest = serde_json::from_str(&manifest_text)?;
    let asset = select_asset(&manifest)
        .ok_or_else(|| anyhow!("no tunnel-client asset for current platform"))?;

    if asset.sha256.trim().is_empty() {
        bail!("manifest asset sha256 is required");
    }

    let current = current_version(settings);

    Ok(TunnelClientVersionStatus {
        installed: settings
            .tunnel_client_path
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty() && Path::new(value).exists()),
        update_available: current.as_deref() != Some(manifest.version.as_str()),
        current_version: current,
        latest_version: Some(manifest.version),
        asset_url: Some(asset.url),
        asset_sha256: Some(asset.sha256),
        checksum_verified: false,
    })
}

/// Derive the current version from `settings`, preferring the explicit
/// `tunnel_client_version` field (which `install_tunnel_client` writes
/// after a successful sha256-verified download). Falls back to the
/// parent directory segment of `tunnel_client_path` only when that
/// path matches the managed `<bin_root>/<version>/tunnel-client[.exe]`
/// layout.
pub fn current_version(settings: &TunnelSettings) -> Option<String> {
    settings
        .tunnel_client_version
        .as_ref()
        .and_then(non_empty)
        .or_else(|| {
            settings
                .tunnel_client_path
                .as_ref()
                .and_then(version_from_managed_path)
        })
}

/// Extract the version segment from a managed binary path of the
/// shape `<bin_root>/<version>/tunnel-client[.exe]`.
///
/// Returns `None` when the path does not match that layout, for
/// example because the file name is not `tunnel-client` or the parent
/// directory name is `"bin"` (the `bin_root` itself).
pub fn version_from_managed_path(path: impl AsRef<str>) -> Option<String> {
    let path = Path::new(path.as_ref());

    let file_name = path.file_name()?.to_str()?;
    if file_name != binary_file_name() {
        return None;
    }

    let version = path.parent()?.file_name()?.to_str()?;
    if version == "bin" || version.trim().is_empty() {
        return None;
    }

    Some(version.to_string())
}

/// Find the highest previously installed version directory under
/// `<app_data_dir>/bin/` that is not `current_version`.
///
/// Only versions that actually contain a `tunnel-client` binary are
/// considered, so an interrupted install cannot poison the rollback
/// candidate list. Versions are compared with a semver-aware ordering
/// so `0.10.0` correctly sorts after `0.2.0`.
pub fn previous_version_dir(
    app_data_dir: &Path,
    current_version: &str,
) -> anyhow::Result<Option<PathBuf>> {
    let bin_root = app_data_dir.join("bin");
    if !bin_root.exists() {
        return Ok(None);
    }

    let mut versions = fs::read_dir(&bin_root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(ToString::to_string))
        .filter(|version| version != current_version)
        .filter(|version| binary_path_for_version(&bin_root, version).exists())
        .collect::<Vec<_>>();

    versions.sort_by(|left, right| compare_versions(left, right));

    Ok(versions.pop().map(|version| bin_root.join(version)))
}

/// Switch the configured `tunnel_client_path` to the binary under the
/// highest previously installed version directory, and persist the
/// new path plus its derived version into `settings`.
///
/// Returns an error if the candidate binary is missing so the
/// caller never ends up pointing at a non-existent file.
pub fn rollback_tunnel_client(
    app_data_dir: &Path,
    settings: &mut TunnelSettings,
) -> anyhow::Result<TunnelClientVersionStatus> {
    let current_version = current_version(settings);

    let Some(current_version) = current_version else {
        return Ok(TunnelClientVersionStatus {
            installed: false,
            current_version: None,
            latest_version: None,
            update_available: false,
            asset_url: None,
            asset_sha256: None,
            checksum_verified: false,
        });
    };

    let Some(previous_dir) = previous_version_dir(app_data_dir, &current_version)? else {
        return Ok(TunnelClientVersionStatus {
            installed: settings
                .tunnel_client_path
                .as_ref()
                .is_some_and(|path| Path::new(path).exists()),
            current_version: Some(current_version),
            latest_version: None,
            update_available: false,
            asset_url: None,
            asset_sha256: None,
            checksum_verified: false,
        });
    };

    let binary = previous_dir.join(binary_file_name());
    if !binary.exists() {
        bail!("rollback binary does not exist: {}", binary.display());
    }

    let new_path = binary.to_string_lossy().to_string();
    let new_version = version_from_managed_path(&new_path)
        .ok_or_else(|| anyhow!("rollback binary path does not contain a managed version"))?;

    settings.tunnel_client_path = Some(new_path);
    settings.tunnel_client_version = Some(new_version.clone());

    Ok(TunnelClientVersionStatus {
        installed: true,
        current_version: Some(new_version),
        latest_version: None,
        update_available: false,
        asset_url: None,
        asset_sha256: None,
        checksum_verified: false,
    })
}

fn http_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(Into::into)
}

fn non_empty(value: impl AsRef<str>) -> Option<String> {
    let trimmed = value.as_ref().trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn binary_path_for_version(bin_root: &Path, version: &str) -> PathBuf {
    bin_root.join(version).join(binary_file_name())
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = parse_version_parts(left);
    let right_parts = parse_version_parts(right);

    left_parts.cmp(&right_parts).then_with(|| left.cmp(right))
}

fn parse_version_parts(value: &str) -> Vec<u64> {
    value
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}
