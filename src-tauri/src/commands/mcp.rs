use crate::product::mcp::server::McpServerManager;
use crate::product::status::McpServerStatus;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn start_mcp_server(
    manager: State<'_, McpServerManager>,
) -> Result<McpServerStatus, String> {
    let home = dirs::home_dir().ok_or_else(|| "cannot resolve home directory".to_string())?;
    let roots: Vec<PathBuf> = vec![home.join("Documents")];
    manager.start(roots).await.map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_mcp_server(manager: State<McpServerManager>) -> McpServerStatus {
    manager.stop()
}

#[tauri::command]
pub fn get_mcp_status(manager: State<McpServerManager>) -> McpServerStatus {
    manager.status()
}
