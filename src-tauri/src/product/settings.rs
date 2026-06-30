use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_MCP_SERVER_PORT: u16 = 17891;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelSettings {
    #[serde(default, skip_serializing)]
    pub openai_api_key: Option<String>,
    pub tunnel_id: Option<String>,
    pub tunnel_client_path: Option<String>,
    pub tunnel_client_version: Option<String>,
    pub resource_root: Option<String>,
    pub mcp_server_port: u16,
    pub log_level: LogLevel,
    pub auto_start: bool,
    pub auto_update_tunnel_client: bool,
}

impl Default for TunnelSettings {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            tunnel_id: None,
            tunnel_client_path: None,
            tunnel_client_version: None,
            resource_root: None,
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            log_level: LogLevel::Info,
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
    pub tunnel_client_version: Option<String>,
    pub resource_root: Option<String>,
    pub mcp_server_port: u16,
    pub log_level: LogLevel,
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

        let normalized = normalize_settings_for_save(next);

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
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_port(value: u16) -> u16 {
    if value == 0 {
        DEFAULT_MCP_SERVER_PORT
    } else {
        value
    }
}

fn normalize_loaded_settings(settings: TunnelSettings) -> TunnelSettings {
    TunnelSettings {
        openai_api_key: normalize_optional_string(settings.openai_api_key),
        tunnel_id: normalize_optional_string(settings.tunnel_id),
        tunnel_client_path: normalize_optional_string(settings.tunnel_client_path),
        tunnel_client_version: normalize_optional_string(settings.tunnel_client_version),
        resource_root: normalize_optional_string(settings.resource_root),
        mcp_server_port: normalize_port(settings.mcp_server_port),
        log_level: settings.log_level,
        auto_start: settings.auto_start,
        auto_update_tunnel_client: settings.auto_update_tunnel_client,
    }
}

fn normalize_settings_for_save(settings: TunnelSettings) -> TunnelSettings {
    let mut normalized = normalize_loaded_settings(settings);
    normalized.openai_api_key = None;
    normalized
}

pub fn to_public_settings(settings: TunnelSettings) -> PublicTunnelSettings {
    let normalized = normalize_loaded_settings(settings);
    let has_key = normalized.openai_api_key.is_some();

    PublicTunnelSettings {
        tunnel_id: normalized.tunnel_id,
        tunnel_client_path: normalized.tunnel_client_path,
        tunnel_client_version: normalized.tunnel_client_version,
        resource_root: normalized.resource_root,
        mcp_server_port: normalized.mcp_server_port,
        log_level: normalized.log_level,
        auto_start: normalized.auto_start,
        auto_update_tunnel_client: normalized.auto_update_tunnel_client,
        openai_api_key_masked: normalized.openai_api_key.as_deref().and_then(mask_secret),
        has_openai_api_key: has_key,
    }
}

pub fn mask_secret(value: &str) -> Option<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return None;
    }

    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() <= 8 {
        return Some("\u{2022}\u{2022}\u{2022}\u{2022}".to_string());
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars.iter().skip(chars.len().saturating_sub(4)).collect();

    Some(format!("{prefix}\u{2022}\u{2022}\u{2022}\u{2022}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::{mask_secret, to_public_settings, LogLevel, SettingsStore, TunnelSettings};
    use tempfile::tempdir;

    #[test]
    fn mask_secret_hides_value() {
        assert_eq!(mask_secret(""), None);
        assert_eq!(mask_secret("   "), None);
        assert_eq!(
            mask_secret("abc"),
            Some("\u{2022}\u{2022}\u{2022}\u{2022}".to_string())
        );
        assert_eq!(
            mask_secret("sk-1234567890abcd"),
            Some(format!("sk-1\u{2022}\u{2022}\u{2022}\u{2022}abcd"))
        );
    }

    #[test]
    fn mask_secret_handles_non_ascii_without_panicking() {
        assert_eq!(
            mask_secret("\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}"),
            Some(format!(
                "\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}\u{2022}\u{2022}\u{2022}\u{2022}\u{5bc6}\u{94a5}\u{5bc6}\u{94a5}"
            ))
        );
    }

    #[test]
    fn public_settings_never_exposes_plain_key() {
        let public = to_public_settings(TunnelSettings {
            openai_api_key: Some("sk-1234567890abcd".to_string()),
            tunnel_id: Some("tun_1".to_string()),
            tunnel_client_path: None,
            tunnel_client_version: None,
            resource_root: Some("/tmp/project".to_string()),
            mcp_server_port: 17891,
            log_level: LogLevel::Info,
            auto_start: true,
            auto_update_tunnel_client: true,
        });

        assert!(public.has_openai_api_key);
        assert_eq!(
            public.openai_api_key_masked,
            Some(format!("sk-1\u{2022}\u{2022}\u{2022}\u{2022}abcd"))
        );
    }

    #[test]
    fn store_loads_default_when_missing() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        assert_eq!(store.load().unwrap(), TunnelSettings::default());
    }

    #[test]
    fn store_saves_and_loads_settings_without_key() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));

        let settings = TunnelSettings {
            openai_api_key: Some("sk-test".to_string()),
            tunnel_id: Some("tun_test".to_string()),
            tunnel_client_path: Some("/tmp/tunnel-client".to_string()),
            tunnel_client_version: Some("0.2.0".to_string()),
            resource_root: Some("/tmp/project".to_string()),
            mcp_server_port: 18888,
            log_level: LogLevel::Debug,
            auto_start: true,
            auto_update_tunnel_client: false,
        };

        store.save(settings.clone()).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.openai_api_key, None);
        assert_eq!(loaded.tunnel_id, settings.tunnel_id);
        assert_eq!(loaded.tunnel_client_path, settings.tunnel_client_path);
        assert_eq!(loaded.tunnel_client_version, settings.tunnel_client_version);
        assert_eq!(loaded.resource_root, settings.resource_root);
        assert_eq!(loaded.mcp_server_port, settings.mcp_server_port);
        assert_eq!(loaded.log_level, settings.log_level);
        assert_eq!(loaded.auto_start, settings.auto_start);
        assert_eq!(
            loaded.auto_update_tunnel_client,
            settings.auto_update_tunnel_client
        );
    }

    #[test]
    fn store_never_writes_openai_key_to_json() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));

        store
            .save(TunnelSettings {
                openai_api_key: Some("sk-secret".to_string()),
                tunnel_id: Some("tun_test".to_string()),
                tunnel_client_path: None,
                tunnel_client_version: None,
                resource_root: None,
                mcp_server_port: 17891,
                log_level: LogLevel::Info,
                auto_start: false,
                auto_update_tunnel_client: true,
            })
            .unwrap();

        let raw = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
        assert!(!raw.contains("sk-secret"));
        assert!(!raw.contains("sk-test"));
        assert_eq!(store.load().unwrap().openai_api_key, None);
    }
}
