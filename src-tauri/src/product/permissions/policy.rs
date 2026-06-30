use super::scope::{PermissionAccess, PermissionDecision, PermissionKind, PermissionScope};
use anyhow::anyhow;
use globset::{Glob, GlobSet, GlobSetBuilder};
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

pub fn sensitive_globs() -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in [
        "**/.ssh/**",
        "**/.gnupg/**",
        "**/Library/Keychains/**",
        "**/.aws/**",
        "**/.kube/**",
        "**/.docker/**",
        "**/.env",
        "**/.env.*",
        "**/id_rsa",
        "**/id_ed25519",
        "**/AppData/Roaming/Microsoft/Credentials/**",
    ] {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

pub fn pattern_to_glob(pattern: &str) -> anyhow::Result<Glob> {
    let expanded = expand_home(pattern);
    let normalized = expanded.replace('\\', "/");
    Glob::new(&normalized).map_err(Into::into)
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
        let target = strip_verbatim_prefix(&canonical)
            .to_string_lossy()
            .replace('\\', "/");

        if self.denylist.is_match(&target) {
            return PermissionDecision {
                allowed: false,
                require_approval: false,
                reason: DENY_REASON_SENSITIVE_PATH.to_string(),
            };
        }

        for scope in &self.scopes {
            if scope.kind != PermissionKind::Filesystem || !scope.access.allows(&requested) {
                continue;
            }
            let Ok(glob) = pattern_to_glob(&scope.pattern) else {
                continue;
            };
            let Ok(set) = GlobSetBuilder::new().add(glob).build() else {
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

        PermissionDecision {
            allowed: false,
            require_approval: false,
            reason: DENY_REASON_NOT_AUTHORIZED.to_string(),
        }
    }
}
