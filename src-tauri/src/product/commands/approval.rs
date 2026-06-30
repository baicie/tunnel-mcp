#![allow(dead_code)]

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandPolicy {
    pub allowed_commands: Vec<String>,
    pub require_approval: bool,
    pub dry_run_first: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandRunMode {
    Explain,
    DryRun,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunRequest {
    pub command: String,
    pub args: Vec<String>,
    pub mode: CommandRunMode,
    pub approval_id: Option<String>,
}

impl Default for CommandPolicy {
    fn default() -> Self {
        Self {
            allowed_commands: vec![
                "npm".to_string(),
                "pnpm".to_string(),
                "cargo".to_string(),
                "git".to_string(),
            ],
            require_approval: true,
            dry_run_first: true,
        }
    }
}

pub fn validate_command(policy: &CommandPolicy, command: &str) -> anyhow::Result<()> {
    validate_policy(policy)?;

    let command = command.trim();

    if command.is_empty() {
        bail!("command is required");
    }

    // Block path separators to prevent `pnpm /bin/sh -c rm -rf /` bypasses
    if command.contains('/') || command.contains('\\') {
        bail!("command path is not allowed: {command}");
    }

    if !policy.allowed_commands.iter().any(|item| item == command) {
        bail!("command is not allowed: {command}");
    }

    Ok(())
}

pub fn validate_command_run(
    policy: &CommandPolicy,
    request: &CommandRunRequest,
) -> anyhow::Result<()> {
    validate_command(policy, &request.command)?;

    // Reject arguments containing null bytes which could inject options
    if request.args.iter().any(|arg| arg.contains('\0')) {
        bail!("command argument contains invalid null byte");
    }

    match request.mode {
        CommandRunMode::Explain | CommandRunMode::DryRun => Ok(()),
        CommandRunMode::Execute => {
            if request
                .approval_id
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                bail!("approved command execution requires approval id");
            }
            Ok(())
        }
    }
}

fn validate_policy(policy: &CommandPolicy) -> anyhow::Result<()> {
    if !policy.require_approval {
        bail!("command approval cannot be disabled in MVP extension stage");
    }

    if !policy.dry_run_first {
        bail!("dry-run / explain first cannot be disabled in MVP extension stage");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_command, validate_command_run, CommandPolicy, CommandRunMode, CommandRunRequest,
    };

    #[test]
    fn allows_whitelisted_command_with_approval() {
        let policy = CommandPolicy::default();
        assert!(validate_command(&policy, "pnpm").is_ok());
        assert!(validate_command(&policy, "cargo").is_ok());
    }

    #[test]
    fn rejects_non_whitelisted_command() {
        let policy = CommandPolicy::default();
        assert!(validate_command(&policy, "rm").is_err());
        assert!(validate_command(&policy, "curl").is_err());
    }

    #[test]
    fn approval_cannot_be_disabled() {
        let policy = CommandPolicy {
            require_approval: false,
            ..CommandPolicy::default()
        };
        assert!(validate_command(&policy, "pnpm").is_err());
    }

    #[test]
    fn dry_run_cannot_be_disabled() {
        let policy = CommandPolicy {
            dry_run_first: false,
            ..CommandPolicy::default()
        };
        assert!(validate_command(&policy, "pnpm").is_err());
    }

    #[test]
    fn rejects_command_path_bypass() {
        let policy = CommandPolicy::default();

        assert!(validate_command(&policy, "/bin/pnpm").is_err());
        assert!(validate_command(&policy, "C:\\Windows\\System32\\cmd.exe").is_err());
        assert!(validate_command(&policy, "./pnpm").is_err());
    }

    #[test]
    fn execute_requires_approval_id() {
        let policy = CommandPolicy::default();

        let request = CommandRunRequest {
            command: "pnpm".to_string(),
            args: vec!["test".to_string()],
            mode: CommandRunMode::Execute,
            approval_id: None,
        };

        assert!(validate_command_run(&policy, &request).is_err());
    }

    #[test]
    fn explain_without_approval_is_allowed() {
        let policy = CommandPolicy::default();

        let request = CommandRunRequest {
            command: "pnpm".to_string(),
            args: vec!["test".to_string()],
            mode: CommandRunMode::Explain,
            approval_id: None,
        };

        assert!(validate_command_run(&policy, &request).is_ok());
    }

    #[test]
    fn dry_run_without_approval_is_allowed() {
        let policy = CommandPolicy::default();

        let request = CommandRunRequest {
            command: "pnpm".to_string(),
            args: vec!["test".to_string()],
            mode: CommandRunMode::DryRun,
            approval_id: None,
        };

        assert!(validate_command_run(&policy, &request).is_ok());
    }

    #[test]
    fn execute_with_valid_approval_is_allowed() {
        let policy = CommandPolicy::default();

        let request = CommandRunRequest {
            command: "pnpm".to_string(),
            args: vec!["test".to_string()],
            mode: CommandRunMode::Execute,
            approval_id: Some("approval-123".to_string()),
        };

        assert!(validate_command_run(&policy, &request).is_ok());
    }

    #[test]
    fn null_byte_in_args_rejected() {
        let policy = CommandPolicy::default();

        let request = CommandRunRequest {
            command: "pnpm".to_string(),
            args: vec!["test\0--dangerous".to_string()],
            mode: CommandRunMode::Execute,
            approval_id: Some("approval-123".to_string()),
        };

        assert!(validate_command_run(&policy, &request).is_err());
    }
}
