use crate::error::ShellError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use super::paths::settings_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellSettings {
    pub theme: ThemeMode,
    pub start_minimized: bool,
}

impl Default for ShellSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            start_minimized: false,
        }
    }
}

pub fn load_settings(app: &AppHandle) -> Result<ShellSettings, ShellError> {
    let path = settings_path(app).map_err(ShellError::Io)?;
    load_settings_from_path(&path)
}

pub fn save_settings(app: &AppHandle, settings: &ShellSettings) -> Result<(), ShellError> {
    let path = settings_path(app).map_err(ShellError::Io)?;
    save_settings_to_path(&path, settings)
}

pub fn load_settings_from_path(path: &Path) -> Result<ShellSettings, ShellError> {
    if !path.exists() {
        return Ok(ShellSettings::default());
    }

    let content = fs::read_to_string(path)?;
    let settings = serde_json::from_str::<ShellSettings>(&content)?;

    Ok(settings)
}

pub fn save_settings_to_path(path: &Path, settings: &ShellSettings) -> Result<(), ShellError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::brand::APP_BRAND;
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn should_return_default_settings_when_missing() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("shell-settings.json");

        let settings = load_settings_from_path(&path).expect("load settings");

        assert_eq!(settings, ShellSettings::default());
    }

    #[test]
    fn should_save_and_load_settings() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("shell-settings.json");

        let settings = ShellSettings {
            theme: ThemeMode::Dark,
            start_minimized: true,
        };

        save_settings_to_path(&path, &settings).expect("save settings");
        let loaded = load_settings_from_path(&path).expect("load settings");

        assert_eq!(loaded, settings);
    }

    #[test]
    fn should_error_when_json_is_invalid() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("shell-settings.json");

        fs::write(&path, "{ invalid json").expect("write invalid json");

        assert!(load_settings_from_path(&path).is_err());
    }

    #[test]
    fn should_create_parent_dir_when_saving() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("nested").join("shell-settings.json");

        save_settings_to_path(&path, &ShellSettings::default()).expect("save settings");

        assert!(path.exists());
    }

    #[test]
    fn settings_file_name_should_come_from_brand() {
        assert_eq!(APP_BRAND.settings_file_name, "shell-settings.json");
    }
}
