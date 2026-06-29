use super::client_binary::{binary_file_name, select_asset, TunnelClientManifest};
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallTunnelClientInput {
    pub manifest_url: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledTunnelClient {
    pub path: String,
    pub version: String,
}

pub async fn install_tunnel_client(
    app_data_dir: &Path,
    input: InstallTunnelClientInput,
) -> anyhow::Result<InstalledTunnelClient> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("build http client")?;

    let manifest_text = client
        .get(&input.manifest_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let manifest: TunnelClientManifest = serde_json::from_str(&manifest_text)?;

    if let Some(expected_version) = input.version.as_ref() {
        if expected_version != &manifest.version {
            return Err(anyhow!(
                "manifest version mismatch: expected {}, got {}",
                expected_version,
                manifest.version
            ));
        }
    }

    let asset = select_asset(&manifest)
        .ok_or_else(|| anyhow!("no tunnel-client asset for current platform"))?;

    let bytes = client
        .get(&asset.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    verify_sha256(&bytes, &asset.sha256)?;

    let bin_dir = app_data_dir.join("bin").join(&manifest.version);
    fs::create_dir_all(&bin_dir).context("create bin directory")?;

    let bin_path = bin_dir.join(binary_file_name());
    let tmp_path = bin_dir.join(format!("{}.download", binary_file_name()));

    {
        let mut file = fs::File::create(&tmp_path).context("create temporary binary file")?;
        file.write_all(&bytes).context("write binary contents")?;
        file.sync_all().context("sync binary contents")?;
    }

    set_executable(&tmp_path)?;

    if bin_path.exists() {
        fs::remove_file(&bin_path).context("remove existing binary before replace")?;
    }

    fs::rename(&tmp_path, &bin_path).context("move verified binary into place")?;

    Ok(InstalledTunnelClient {
        path: bin_path.to_string_lossy().to_string(),
        version: manifest.version,
    })
}

pub fn verify_sha256(bytes: &[u8], expected: &str) -> anyhow::Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected.to_lowercase() {
        return Err(anyhow!("sha256 mismatch"));
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).context("set executable permission")
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
