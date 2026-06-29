use tunnel_mcp::shell::runtime_boundary::SHELL_ALLOWED_COMMANDS;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(relative: &str) -> String {
    let path = manifest_dir().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn extract_generate_handler_commands(source: &str) -> Vec<String> {
    let marker = "tauri::generate_handler![";
    let start = source
        .find(marker)
        .expect("lib.rs should contain tauri::generate_handler![...]")
        + marker.len();

    let rest = &source[start..];
    let end = rest
        .find(']')
        .expect("generate_handler! should have a closing bracket");

    rest[..end]
        .lines()
        .map(|line| line.split("//").next().unwrap_or_default().trim())
        .filter(|line| !line.is_empty())
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            entry
                .rsplit("::")
                .next()
                .expect("command path should contain a function name")
                .trim()
                .to_string()
        })
        .collect()
}

#[test]
fn lib_rs_generate_handler_should_match_allowed_shell_commands_exactly() {
    let source = read_source("src/lib.rs");
    let commands = extract_generate_handler_commands(&source);
    let allowed = SHELL_ALLOWED_COMMANDS
        .iter()
        .map(|command| command.to_string())
        .collect::<Vec<_>>();

    assert_eq!(commands, allowed);
}

#[test]
fn lib_rs_should_not_export_legacy_business_modules() {
    let source = read_source("src/lib.rs").to_lowercase();

    for forbidden in [
        "pub use app_config",
        "pub use codex_config",
        "pub use commands::*",
        "pub use config",
        "pub use database",
        "pub use deeplink",
        "pub use mcp",
        "pub use provider",
        "pub use services",
        "pub use settings",
        "pub use store",
        "pub use app_store",
        "pub use auto_launch",
        "pub use claude_",
        "pub use gemini_",
        "pub use hermes_",
        "pub use openclaw_",
        "pub use opencode_",
        "pub use prompt",
        "pub use proxy",
        "pub use session_manager",
        "pub use lightweight",
        "pub use s3_sync",
        "pub use usage",
        "pub use webdav",
        "pub use workspace",
        "pub use update_settings",
        "pub use app_settings",
        "pub use app_state",
    ] {
        assert!(
            !source.contains(forbidden),
            "lib.rs still contains legacy export: {forbidden}"
        );
    }
}

#[test]
fn lib_rs_should_only_mount_shell_commands() {
    let source = read_source("src/lib.rs");

    for command in [
        "get_app_info",
        "open_external",
        "get_settings",
        "save_settings",
        "update_tray_menu",
    ] {
        assert!(source.contains(command), "missing shell command: {command}");
    }

    let lower = source.to_lowercase();
    for forbidden in [
        "provider",
        "proxy",
        "mcp",
        "prompt",
        "skills",
        "usage",
        "webdav",
        "codex",
        "gemini",
        "claude",
        "openclaw",
        "opencode",
        "hermes",
        "subscription",
        "balance",
        "workspace",
        "copilot",
        "global_proxy",
        "failover",
    ] {
        assert!(
            !lower.contains(forbidden),
            "lib.rs contains old business marker: {forbidden}"
        );
    }
}

#[test]
fn commands_mod_should_only_expose_shell_commands() {
    let source = read_source("src/commands/mod.rs").to_lowercase();

    for required in ["pub mod app;", "pub mod settings;", "pub mod shell;"] {
        assert!(
            source.contains(required),
            "missing command module: {required}"
        );
    }

    for forbidden in [
        "auth",
        "balance",
        "codex_oauth",
        "coding_plan",
        "config",
        "copilot",
        "deeplink",
        "env",
        "failover",
        "global_proxy",
        "hermes",
        "import_export",
        "mcp",
        "misc",
        "model_fetch",
        "omo",
        "openclaw",
        "plugin",
        "prompt",
        "provider",
        "proxy",
        "session_manager",
        "skill",
        "stream_check",
        "subscription",
        "sync_support",
        "lightweight",
        "s3_sync",
        "usage",
        "webdav_sync",
        "workspace",
    ] {
        assert!(
            !source.contains(forbidden),
            "commands/mod.rs contains old command module: {forbidden}"
        );
    }
}

#[test]
fn lib_rs_run_function_should_not_trigger_business_setup() {
    let source = read_source("src/lib.rs").to_lowercase();

    let body_start = source
        .find("pub fn run()")
        .expect("lib.rs should contain pub fn run()");
    let after_signature = &source[body_start..];
    let open = after_signature
        .find('{')
        .expect("run() should have an opening brace");
    let body = &after_signature[open..];

    for forbidden in [
        "migrate_skills_to_ssot",
        "migrate_codex_history",
        "sync_enabled_to_codex",
        "sync_enabled_to_claude",
        "sync_enabled_to_gemini",
        "import_from_codex",
        "import_from_claude",
        "import_from_gemini",
        "trigger_usage_sync",
        "trigger_webdav_sync",
        "trigger_s3_sync",
        "switch_provider",
        "start_proxy",
        "stop_proxy",
        "import_mcp_servers",
    ] {
        assert!(
            !body.contains(forbidden),
            "lib.rs::run() body should not reference legacy business call: {forbidden}"
        );
    }
}
