// Desktop shell runtime entrypoint.
//
// The shell only exposes a minimal Tauri command set plus a tray
// no-op. Product-specific commands live under the `commands` module
// as adapters that delegate to the product layer in `crate::product`.
// They are still wired into `tauri::generate_handler!` here, with the
// full command surface validated by `assert_runtime_boundary` so that
// drift between the registered list and the design contract is caught
// at startup.
mod commands;
pub mod error;
mod product;
pub mod shell;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    shell::panic_hook::install_panic_hook();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build());

    let builder = shell::logging::plugin(builder);
    let builder = shell::updater::plugin(builder);

    builder
        .manage(product::mcp::server::McpServerManager::default())
        .manage(product::tunnel::client_process::TunnelProcessManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::app::open_external,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::shell::update_tray_menu,
            commands::tunnel::get_tunnel_settings,
            commands::tunnel::save_tunnel_settings,
            commands::tunnel::get_tunnel_status,
            commands::mcp::start_mcp_server,
            commands::mcp::stop_mcp_server,
            commands::mcp::get_mcp_status,
            commands::tunnel_process::install_tunnel_client,
            commands::tunnel_process::start_tunnel_client,
            commands::tunnel_process::stop_tunnel_client,
            commands::tunnel_process::restart_tunnel_client,
            commands::tunnel_process::get_tunnel_client_logs,
            commands::permissions::list_permission_scopes,
            commands::permissions::add_permission_scope,
            commands::permissions::remove_permission_scope,
            commands::permissions::check_permission,
            commands::approvals::list_approval_requests,
            commands::approvals::approve_request,
            commands::approvals::reject_request,
        ])
        .setup(|app| {
            shell::runtime_boundary::assert_runtime_boundary(
                shell::runtime_boundary::registered_command_names().as_slice(),
            )
            .map_err(Box::<dyn std::error::Error>::from)?;

            shell::tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running desktop shell");
}
