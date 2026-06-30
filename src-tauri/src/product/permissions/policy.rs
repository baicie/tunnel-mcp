use super::scope::{PermissionAccess, PermissionDecision, PermissionKind, PermissionScope};
use anyhow::{anyhow, Context};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::warn;
use std::path::{Path, PathBuf};

pub const DENY_REASON_SENSITIVE_PATH: &str = "sensitive path denied";
pub const DENY_REASON_NOT_AUTHORIZED: &str = "path is not authorized";

pub fn expand_home(pattern: &str) -> String {
    if let Some(rest) = pattern.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }

    pattern.to_string()
}

pub fn normalize_for_match(path: &Path) -> anyhow::Result<PathBuf> {
    path.canonicalize()
        .map_err(|err| anyhow!("invalid path: {}", err))
}

fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let text = path.to_string_lossy().to_string();
    let trimmed = text
        .strip_prefix(r"\\?\")
        .map(str::to_string)
        .unwrap_or(text);

    PathBuf::from(trimmed)
}

pub fn normalize_path_text(path: &Path) -> String {
    strip_verbatim_prefix(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn sensitive_globs() -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for pattern in [
        "**/.ssh",
        "**/.ssh/**",
        "**/.gnupg",
        "**/.gnupg/**",
        "**/Library/Keychains",
        "**/Library/Keychains/**",
        "**/.aws",
        "**/.aws/**",
        "**/.kube",
        "**/.kube/**",
        "**/.docker",
        "**/.docker/**",
        "**/.env",
        "**/.env.*",
        "**/id_rsa",
        "**/id_ed25519",
        "**/AppData/Roaming/Microsoft/Credentials",
        "**/AppData/Roaming/Microsoft/Credentials/**",
    ] {
        builder.add(Glob::new(pattern)?);
    }

    Ok(builder.build()?)
}

fn glob_meta_offset(value: &str) -> Option<usize> {
    value
        .as_bytes()
        .iter()
        .position(|byte| matches!(byte, b'*' | b'?' | b'[' | b'{'))
}

pub fn normalize_scope_pattern(pattern: &str) -> anyhow::Result<String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("permission pattern is required"));
    }

    let expanded = expand_home(trimmed);
    let slash_normalized = expanded.replace('\\', "/");

    if let Some(idx) = glob_meta_offset(&slash_normalized) {
        let prefix = &slash_normalized[..idx];
        let suffix = &slash_normalized[idx..];

        let normalized_prefix = prefix_canonicalized_or_raw(prefix);
        let combined = format!("{normalized_prefix}{suffix}");

        Glob::new(&combined).context("invalid permission glob")?;
        return Ok(combined);
    }

    if let Ok(canonical) = PathBuf::from(&expanded).canonicalize() {
        if canonical.is_dir() {
            return Ok(format!("{}/**", normalize_path_text(&canonical)));
        }
        return Ok(normalize_path_text(&canonical));
    }

    Ok(slash_normalized)
}

fn prefix_canonicalized_or_raw(prefix: &str) -> String {
    let trimmed = prefix.trim_end_matches('/');

    if trimmed.is_empty() {
        return String::new();
    }

    match PathBuf::from(trimmed).canonicalize() {
        Ok(canonical) => format!("{}/", normalize_path_text(&canonical)),
        Err(_) => format!("{prefix}/"),
    }
}

pub fn pattern_to_glob(pattern: &str) -> anyhow::Result<Glob> {
    Glob::new(pattern).map_err(Into::into)
}

#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    scopes: Vec<PermissionScope>,
    denylist: GlobSet,
}

impl PermissionPolicy {
    pub fn new(scopes: Vec<PermissionScope>) -> anyhow::Result<Self> {
        Ok(Self {
            scopes,
            denylist: sensitive_globs()?,
        })
    }

    pub fn check_path(&self, path: &Path, requested: PermissionAccess) -> PermissionDecision {
        let canonical = match normalize_for_match(path) {
            Ok(value) => value,
            Err(_) => {
                return PermissionDecision {
                    allowed: false,
                    require_approval: false,
                    reason: "invalid path".to_string(),
                };
            }
        };

        let target = normalize_path_text(&canonical);

        if self.denylist.is_match(&target) {
            warn!(
                "permission denied: reason={} path={}",
                DENY_REASON_SENSITIVE_PATH, target
            );

            return PermissionDecision {
                allowed: false,
                require_approval: false,
                reason: DENY_REASON_SENSITIVE_PATH.to_string(),
            };
        }

        for scope in &self.scopes {
            if scope.kind != PermissionKind::Filesystem {
                continue;
            }

            if !scope.access.allows(&requested) {
                continue;
            }

            let Ok(pattern) = normalize_scope_pattern(&scope.pattern) else {
                continue;
            };

            let Ok(glob) = pattern_to_glob(&pattern) else {
                continue;
            };

            let mut builder = GlobSetBuilder::new();
            builder.add(glob);

            let Ok(set) = builder.build() else {
                continue;
            };

            if set.is_match(&target) {
                let require_approval =
                    scope.require_approval || requested != PermissionAccess::Read;

                return PermissionDecision {
                    allowed: true,
                    require_approval,
                    reason: "matched permission scope".to_string(),
                };
            }
        }

        warn!(
            "permission denied: reason={} path={}",
            DENY_REASON_NOT_AUTHORIZED, target
        );

        PermissionDecision {
            allowed: false,
            require_approval: false,
            reason: DENY_REASON_NOT_AUTHORIZED.to_string(),
        }
    }

    /// Check a write target that may not exist yet. For new files the
    /// policy is evaluated against the canonicalized parent directory
    /// joined with the file name, so creating a file inside an allowed
    /// scope does not require the file to pre-exist.
    pub fn check_write_target(&self, path: &Path) -> PermissionDecision {
        let target = if path.exists() {
            match path.canonicalize() {
                Ok(value) => value,
                Err(_) => {
                    return PermissionDecision {
                        allowed: false,
                        require_approval: false,
                        reason: "invalid path".to_string(),
                    };
                }
            }
        } else {
            let Some(parent) = path.parent() else {
                return PermissionDecision {
                    allowed: false,
                    require_approval: false,
                    reason: "invalid path".to_string(),
                };
            };

            let Ok(parent) = parent.canonicalize() else {
                return PermissionDecision {
                    allowed: false,
                    require_approval: false,
                    reason: "invalid parent path".to_string(),
                };
            };

            let Some(file_name) = path.file_name() else {
                return PermissionDecision {
                    allowed: false,
                    require_approval: false,
                    reason: "invalid path".to_string(),
                };
            };

            parent.join(file_name)
        };

        self.check_path(&target, PermissionAccess::Write)
    }
}
