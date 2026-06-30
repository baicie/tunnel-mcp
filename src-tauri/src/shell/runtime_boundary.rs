use crate::error::ShellError;

/// Legacy business keywords blacklisted from the shell layer. Any
/// `src-tauri/src/**` file (excluding the shell runtime boundary
/// itself and the tunnel-mcp product directories) containing these
/// markers is treated by `tests/shell_boundary_test.rs` as residual
/// product code that should not live in the shell runtime.
pub const SHELL_FORBIDDEN_MARKERS: &[&str] = &[
    "provider",
    "proxy",
    "prompt",
    "skills",
    "session_manager",
    "usage",
    "usage_events",
    "usage_script",
    "webdav",
    "s3_sync",
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
];

// Shell-level commands. These are the surface every variant of the
// desktop shell template must ship. The shell alone would never
// produce tunnel / mcp / provider traffic.
pub const SHELL_COMMANDS: &[&str] = &[
    "get_app_info",
    "open_external",
    "get_settings",
    "save_settings",
    "update_tray_menu",
];

// Product commands registered by the Tunnel MCP product crate. New
// product commands should be added here and wired through
// `lib.rs::run` alongside the shell handler list.
pub const PRODUCT_COMMANDS: &[&str] = &[
    "get_tunnel_settings",
    "save_tunnel_settings",
    "get_tunnel_status",
    "start_mcp_server",
    "stop_mcp_server",
    "get_mcp_status",
    "install_tunnel_client",
    "start_tunnel_client",
    "stop_tunnel_client",
    "restart_tunnel_client",
    "get_tunnel_client_logs",
    "list_permission_scopes",
    "add_permission_scope",
    "remove_permission_scope",
    "check_permission",
    "list_approval_requests",
    "approve_request",
    "reject_request",
    "list_logs",
    "export_diagnostics",
    "check_app_update",
    "check_tunnel_client_update",
    "rollback_tunnel_client",
];

/// Combined command surface exposed by `lib.rs`. The runtime bootstrap
/// asserts this slice matches `tauri::generate_handler![...]` so a
/// drift is caught at startup.
pub fn registered_command_names() -> Vec<&'static str> {
    let mut commands = Vec::with_capacity(SHELL_COMMANDS.len() + PRODUCT_COMMANDS.len());
    commands.extend_from_slice(SHELL_COMMANDS);
    commands.extend_from_slice(PRODUCT_COMMANDS);
    commands
}

pub fn assert_runtime_boundary(commands: &[&str]) -> Result<(), ShellError> {
    let expected = registered_command_names();

    let mut missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|name| !commands.contains(name))
        .collect();
    missing.sort();
    missing.dedup();

    if !missing.is_empty() {
        return Err(ShellError::RuntimeBoundary(format!(
            "required commands are missing from the runtime: {}",
            missing.join(", ")
        )));
    }

    let mut unexpected: Vec<&str> = commands
        .iter()
        .copied()
        .filter(|name| !expected.contains(name))
        .collect();
    unexpected.sort();
    unexpected.dedup();

    if !unexpected.is_empty() {
        return Err(ShellError::RuntimeBoundary(format!(
            "commands are not allowed in the runtime: {}",
            unexpected.join(", ")
        )));
    }

    Ok(())
}

/// Backwards-compatible alias for the original shell-only assertion.
/// The shell layer is allowed to register exactly the shell slice.
pub fn assert_shell_runtime_boundary(commands: &[&str]) -> Result<(), ShellError> {
    if commands.len() != SHELL_COMMANDS.len() {
        return Err(ShellError::RuntimeBoundary(
            "shell runtime command count drifted".to_string(),
        ));
    }

    for command in commands {
        if !SHELL_COMMANDS.contains(command) {
            return Err(ShellError::RuntimeBoundary(format!(
                "command `{command}` is not allowed in the shell runtime"
            )));
        }
    }

    for required in SHELL_COMMANDS {
        if !commands.contains(required) {
            return Err(ShellError::RuntimeBoundary(format!(
                "required command `{required}` is missing from the shell runtime"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_commands_match_allowed_list() {
        let commands: Vec<&str> = SHELL_COMMANDS.to_vec();
        assert_shell_runtime_boundary(&commands).expect("valid shell commands");
        assert_eq!(commands.len(), SHELL_COMMANDS.len());
    }

    #[test]
    fn should_reject_unknown_command() {
        let commands = vec![
            "get_app_info",
            "open_external",
            "get_settings",
            "save_settings",
            "update_tray_menu",
            "provider_command",
        ];

        assert!(assert_shell_runtime_boundary(&commands).is_err());
    }

    #[test]
    fn should_reject_missing_command() {
        let commands = vec![
            "get_app_info",
            "open_external",
            "get_settings",
            "save_settings",
        ];

        assert!(assert_shell_runtime_boundary(&commands).is_err());
    }

    #[test]
    fn registered_commands_cover_shell_and_product() {
        let commands: Vec<&str> = registered_command_names();

        assert!(assert_runtime_boundary(&commands).is_ok());
    }
}
