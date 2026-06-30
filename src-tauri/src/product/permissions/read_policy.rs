use super::policy::PermissionPolicy;
use super::scope::{PermissionAccess, PermissionScope};
use crate::product::mcp::resources::ReadPolicy;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PermissionReadPolicy {
    policy: PermissionPolicy,
    authorized_roots: Vec<PathBuf>,
}

impl PermissionReadPolicy {
    pub fn new(scopes: Vec<PermissionScope>) -> anyhow::Result<Self> {
        let authorized_roots = scopes
            .iter()
            .filter(|scope| {
                scope.kind == super::scope::PermissionKind::Filesystem
                    && scope.access.allows(&PermissionAccess::Read)
                    && !scope.require_approval
            })
            .filter_map(|scope| root_from_pattern(&scope.pattern))
            .collect::<Vec<_>>();

        Ok(Self {
            policy: PermissionPolicy::new(scopes)?,
            authorized_roots,
        })
    }
}

impl ReadPolicy for PermissionReadPolicy {
    fn can_read(&self, path: &Path) -> bool {
        let decision = self.policy.check_path(path, PermissionAccess::Read);

        decision.allowed && !decision.require_approval
    }

    fn authorized_roots(&self) -> Vec<PathBuf> {
        self.authorized_roots.clone()
    }
}

fn root_from_pattern(pattern: &str) -> Option<PathBuf> {
    let value = pattern
        .strip_suffix("/**")
        .or_else(|| pattern.strip_suffix("/*"))
        .unwrap_or(pattern);

    let path = PathBuf::from(value);
    path.canonicalize().ok().filter(|value| value.is_dir())
}
