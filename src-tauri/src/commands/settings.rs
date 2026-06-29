use crate::shell::settings_store::{self, ShellSettings};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<ShellSettings, String> {
    settings_store::load_settings(&app).map_err(String::from)
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    settings: ShellSettings,
) -> Result<ShellSettings, String> {
    settings_store::save_settings(&app, &settings).map_err(String::from)?;
    Ok(settings)
}
