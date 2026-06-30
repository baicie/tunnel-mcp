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
    if !policy.allowed_commands.iter().any(|item| item == command) {
        bail!("command is not allowed: {command}");
    }
    if !policy.require_approval {
        bail!("command approval cannot be disabled in MVP extension stage");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_command, CommandPolicy};

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
    fn default_policy_locks_dry_run_first() {
        let policy = CommandPolicy::default();
        assert!(policy.dry_run_first);
        assert!(!policy.allowed_commands.is_empty());
    }
}
