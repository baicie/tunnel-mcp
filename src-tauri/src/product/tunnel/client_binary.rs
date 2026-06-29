use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelClientAsset {
    pub platform: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelClientManifest {
    pub version: String,
    pub assets: Vec<TunnelClientAsset>,
}

pub fn current_platform() -> String {
    match std::env::consts::OS {
        "macos" => "darwin".to_string(),
        "windows" => "windows".to_string(),
        "linux" => "linux".to_string(),
        other => other.to_string(),
    }
}

pub fn current_arch() -> String {
    match std::env::consts::ARCH {
        "aarch64" => "arm64".to_string(),
        "x86_64" => "x64".to_string(),
        other => other.to_string(),
    }
}

pub fn select_asset(manifest: &TunnelClientManifest) -> Option<TunnelClientAsset> {
    let platform = current_platform();
    let arch = current_arch();
    manifest
        .assets
        .iter()
        .find(|asset| asset.platform == platform && asset.arch == arch)
        .cloned()
}

pub fn binary_file_name() -> &'static str {
    if cfg!(windows) {
        "tunnel-client.exe"
    } else {
        "tunnel-client"
    }
}
