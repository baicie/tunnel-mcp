use crate::product::security::secret_store::{
    load_openai_key, save_openai_key, KeyringSecretStore,
};
use crate::product::settings::{
    to_public_settings, PublicTunnelSettings, SettingsError, SettingsStore, TunnelSettings,
};
use crate::product::status::TunnelStatus;
use crate::product::tunnel::client_process::TunnelProcessManager;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|err| err.to_string())
}

fn map_error(error: SettingsError) -> String {
    error.to_string()
}

#[tauri::command]
pub fn get_tunnel_settings(app: AppHandle) -> Result<PublicTunnelSettings, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    let mut settings = store.load().map_err(map_error)?;

    if let Ok(key) = load_openai_key(&KeyringSecretStore) {
        settings.openai_api_key = key;
    }

    Ok(to_public_settings(settings))
}

#[tauri::command]
pub fn save_tunnel_settings(
    app: AppHandle,
    settings: TunnelSettings,
) -> Result<PublicTunnelSettings, String> {
    let store = SettingsStore::new(settings_path(&app)?);

    let key_to_save = settings.openai_api_key.clone();
    let mut settings_without_key = settings;
    settings_without_key.openai_api_key = None;

    let saved = store
        .save(settings_without_key)
        .map_err(map_error)?;

    if let Err(err) = save_openai_key(&KeyringSecretStore, key_to_save) {
        log::warn!("failed to save OpenAI key to keyring: {}", err);
    }

    let mut final_settings = saved;
    if let Ok(key) = load_openai_key(&KeyringSecretStore) {
        final_settings.openai_api_key = key;
    }

    Ok(to_public_settings(final_settings))
}

#[tauri::command]
pub fn get_tunnel_status(
    app: AppHandle,
    manager: State<TunnelProcessManager>,
) -> Result<TunnelStatus, String> {
    let store = SettingsStore::new(settings_path(&app)?);
    let settings = store.load().map_err(map_error)?;
    manager.status(&settings).map_err(|err| err.to_string())
}
