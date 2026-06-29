use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelSettings {
    pub openai_api_key: Option<String>,
    pub tunnel_id: Option<String>,
    pub tunnel_client_path: Option<String>,
    pub auto_start: bool,
    pub auto_update_tunnel_client: bool,
}

impl Default for TunnelSettings {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            tunnel_id: None,
            tunnel_client_path: None,
            auto_start: false,
            auto_update_tunnel_client: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicTunnelSettings {
    pub tunnel_id: Option<String>,
    pub tunnel_client_path: Option<String>,
    pub auto_start: bool,
    pub auto_update_tunnel_client: bool,
    pub openai_api_key_masked: Option<String>,
    pub has_openai_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    tunnel: TunnelSettings,
}

#[derive(Debug)]
pub enum SettingsError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<std::io::Error> for SettingsError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SettingsError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<TunnelSettings, SettingsError> {
        if !self.path.exists() {
            return Ok(TunnelSettings::default());
        }
        let raw = fs::read_to_string(&self.path)?;
        let file: SettingsFile = serde_json::from_str(&raw)?;
        Ok(file.tunnel)
    }

    pub fn save(&self, next: TunnelSettings) -> Result<TunnelSettings, SettingsError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = SettingsFile {
            version: 1,
            tunnel: next.clone(),
        };
        let raw = serde_json::to_string_pretty(&file)?;
        fs::write(&self.path, raw)?;
        Ok(next)
    }
}

pub fn to_public_settings(settings: TunnelSettings) -> PublicTunnelSettings {
    let has_key = settings
        .openai_api_key
        .as_ref()
        .is_some_and(|value| !value.is_empty());
    PublicTunnelSettings {
        tunnel_id: settings.tunnel_id,
        tunnel_client_path: settings.tunnel_client_path,
        auto_start: settings.auto_start,
        auto_update_tunnel_client: settings.auto_update_tunnel_client,
        openai_api_key_masked: settings.openai_api_key.as_deref().and_then(mask_secret),
        has_openai_api_key: has_key,
    }
}

pub fn mask_secret(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    if value.len() <= 8 {
        return Some("••••".to_string());
    }
    Some(format!("{}••••{}", &value[..4], &value[value.len() - 4..]))
}

#[cfg(test)]
mod tests {
    use super::{mask_secret, to_public_settings, SettingsStore, TunnelSettings};
    use tempfile::tempdir;

    #[test]
    fn mask_secret_hides_value() {
        assert_eq!(mask_secret(""), None);
        assert_eq!(mask_secret("abc"), Some("••••".to_string()));
        assert_eq!(
            mask_secret("sk-1234567890abcd"),
            Some("sk-1••••abcd".to_string())
        );
    }

    #[test]
    fn public_settings_never_exposes_plain_key() {
        let public = to_public_settings(TunnelSettings {
            openai_api_key: Some("sk-1234567890abcd".to_string()),
            tunnel_id: Some("tun_1".to_string()),
            tunnel_client_path: None,
            auto_start: true,
            auto_update_tunnel_client: true,
        });

        assert!(public.has_openai_api_key);
        assert_eq!(
            public.openai_api_key_masked,
            Some("sk-1••••abcd".to_string())
        );
    }

    #[test]
    fn store_loads_default_when_missing() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        assert_eq!(store.load().unwrap(), TunnelSettings::default());
    }

    #[test]
    fn store_saves_and_loads_settings() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        let settings = TunnelSettings {
            openai_api_key: Some("sk-test".to_string()),
            tunnel_id: Some("tun_test".to_string()),
            tunnel_client_path: Some("/tmp/tunnel-client".to_string()),
            auto_start: true,
            auto_update_tunnel_client: false,
        };

        store.save(settings.clone()).unwrap();
        assert_eq!(store.load().unwrap(), settings);
    }
}
