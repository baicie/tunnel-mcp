// The desktop shell exposes three Tauri command modules: `app`,
// `settings`, and `shell`. Product-specific commands should not be
// added as `pub use xxx::*;` re-exports here. Integration tests access
// shell behaviour through `pub mod shell`, not through this adapter
// layer.
pub mod app;
pub mod settings;
pub mod shell;
