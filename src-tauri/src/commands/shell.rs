use crate::shell::tray;
use tauri::AppHandle;

#[tauri::command]
pub async fn update_tray_menu(app: AppHandle) -> Result<(), String> {
    tray::update_tray_menu(&app).map_err(|error| error.to_string())
}
