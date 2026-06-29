use crate::error::ShellError;

pub const SHELL_ALLOWED_COMMANDS: &[&str] = &[
    "get_app_info",
    "open_external",
    "get_settings",
    "save_settings",
    "update_tray_menu",
];

/// Legacy business keywords blacklisted from the shell runtime. Any
/// `src-tauri/src/**` file (excluding this module itself) containing
/// these markers is treated by `tests/shell_boundary_test.rs` as
/// residual product code that should not live in the shell runtime.
pub const SHELL_FORBIDDEN_MARKERS: &[&str] = &[
    "provider",
    "proxy",
    "mcp",
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

pub fn registered_command_names() -> Vec<&'static str> {
    SHELL_ALLOWED_COMMANDS.to_vec()
}

pub fn assert_shell_runtime_boundary(commands: &[&str]) -> Result<(), ShellError> {
    for command in commands {
        if !SHELL_ALLOWED_COMMANDS.contains(command) {
            return Err(ShellError::RuntimeBoundary(format!(
                "command `{command}` is not allowed in the shell runtime"
            )));
        }
    }

    for required in SHELL_ALLOWED_COMMANDS {
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
    fn registered_commands_should_match_allowed_commands() {
        let commands = registered_command_names();

        assert_shell_runtime_boundary(commands.as_slice()).expect("valid commands");
        assert_eq!(commands.len(), SHELL_ALLOWED_COMMANDS.len());
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

        assert!(assert_shell_runtime_boundary(commands.as_slice()).is_err());
    }

    #[test]
    fn should_reject_missing_command() {
        let commands = vec![
            "get_app_info",
            "open_external",
            "get_settings",
            "save_settings",
        ];

        assert!(assert_shell_runtime_boundary(commands.as_slice()).is_err());
    }
}
