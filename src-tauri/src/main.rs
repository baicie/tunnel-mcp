// Prevents an additional console window on Windows in release builds.
// DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // On Linux, opt out of WebKitGTK's DMA-BUF renderer and compositor.
    // A small number of driver/compositor combinations (e.g. some
    // Debian/Nvidia setups and certain Wayland compositors) render a
    // blank/black window or break window resize and click handling
    // when those features are enabled.
    // Reference: https://github.com/tauri-apps/tauri/issues/9394
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    desktop_shell::run();
}
