use crate::shell::app_info::{app_info, AppInfo};
use crate::shell::external_url::open_external_url;
use tauri::AppHandle;

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    app_info()
}

#[tauri::command]
pub async fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    open_external_url(&app, &url)
}
