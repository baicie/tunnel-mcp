// Desktop shell runtime entrypoint.
//
// The shell only exposes a minimal Tauri command set plus a tray
// no-op. Product-specific commands must live under their own modules
// and reach `pub mod shell` through pure logic — they should never be
// re-exported from `lib.rs` or wired into the tauri invoke_handler
// directly. Integration tests verify this boundary in
// `tests/runtime_boundary_test.rs` and `tests/shell_boundary_test.rs`.
mod commands;
pub mod error;
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
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::app::open_external,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::shell::update_tray_menu,
        ])
        .setup(|app| {
            shell::runtime_boundary::assert_shell_runtime_boundary(
                shell::runtime_boundary::registered_command_names().as_slice(),
            )
            .map_err(Box::<dyn std::error::Error>::from)?;

            shell::tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running desktop shell");
}
