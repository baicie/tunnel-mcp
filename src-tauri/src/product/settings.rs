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
        Ok(normalize_loaded_settings(file.tunnel))
    }

    pub fn save(&self, next: TunnelSettings) -> Result<TunnelSettings, SettingsError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let existing = if self.path.exists() {
            self.load()?
        } else {
            TunnelSettings::default()
        };
        let normalized = normalize_settings_for_save(next, existing.openai_api_key);

        let file = SettingsFile {
            version: 1,
            tunnel: normalized.clone(),
        };
        let raw = serde_json::to_string_pretty(&file)?;
        fs::write(&self.path, raw)?;
        Ok(normalized)
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_loaded_settings(settings: TunnelSettings) -> TunnelSettings {
    TunnelSettings {
        openai_api_key: normalize_optional_string(settings.openai_api_key),
        tunnel_id: normalize_optional_string(settings.tunnel_id),
        tunnel_client_path: normalize_optional_string(settings.tunnel_client_path),
        auto_start: settings.auto_start,
        auto_update_tunnel_client: settings.auto_update_tunnel_client,
    }
}

fn normalize_settings_for_save(
    settings: TunnelSettings,
    existing_openai_api_key: Option<String>,
) -> TunnelSettings {
    let openai_api_key = normalize_optional_string(settings.openai_api_key)
        .or_else(|| normalize_optional_string(existing_openai_api_key));

    TunnelSettings {
        openai_api_key,
        tunnel_id: normalize_optional_string(settings.tunnel_id),
        tunnel_client_path: normalize_optional_string(settings.tunnel_client_path),
        auto_start: settings.auto_start,
        auto_update_tunnel_client: settings.auto_update_tunnel_client,
    }
}

pub fn to_public_settings(settings: TunnelSettings) -> PublicTunnelSettings {
    let settings = normalize_loaded_settings(settings);
    let has_key = settings
        .openai_api_key
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
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
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    let chars = value.chars().collect::<Vec<_>>();

    if chars.len() <= 8 {
        return Some("••••".to_string());
    }

    let prefix = chars.iter().take(4).collect::<String>();
    let suffix = chars.iter().skip(chars.len() - 4).collect::<String>();
    Some(format!("{prefix}••••{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::{mask_secret, to_public_settings, SettingsStore, TunnelSettings};
    use tempfile::tempdir;

    #[test]
    fn mask_secret_hides_value() {
        assert_eq!(mask_secret(""), None);
        assert_eq!(mask_secret("   "), None);
        assert_eq!(mask_secret("abc"), Some("••••".to_string()));
        assert_eq!(
            mask_secret("sk-1234567890abcd"),
            Some("sk-1••••abcd".to_string())
        );
    }

    #[test]
    fn mask_secret_handles_non_ascii_values() {
        assert_eq!(
            mask_secret("密钥一二三四五六七八九十"),
            Some("密钥一二••••七八九十".to_string())
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

    #[test]
    fn store_preserves_existing_key_when_saved_key_is_blank() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));

        store
            .save(TunnelSettings {
                openai_api_key: Some("sk-existing".to_string()),
                tunnel_id: Some("tun_old".to_string()),
                tunnel_client_path: None,
                auto_start: false,
                auto_update_tunnel_client: true,
            })
            .unwrap();

        let saved = store
            .save(TunnelSettings {
                openai_api_key: Some("   ".to_string()),
                tunnel_id: Some("tun_new".to_string()),
                tunnel_client_path: None,
                auto_start: true,
                auto_update_tunnel_client: false,
            })
            .unwrap();

        assert_eq!(saved.openai_api_key, Some("sk-existing".to_string()));
        assert_eq!(saved.tunnel_id, Some("tun_new".to_string()));
        assert!(saved.auto_start);
        assert!(!saved.auto_update_tunnel_client);
    }

    #[test]
    fn store_normalizes_blank_optional_values() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));

        let saved = store
            .save(TunnelSettings {
                openai_api_key: Some("   ".to_string()),
                tunnel_id: Some("   ".to_string()),
                tunnel_client_path: Some("   ".to_string()),
                auto_start: false,
                auto_update_tunnel_client: true,
            })
            .unwrap();

        assert_eq!(saved.openai_api_key, None);
        assert_eq!(saved.tunnel_id, None);
        assert_eq!(saved.tunnel_client_path, None);
    }
}
