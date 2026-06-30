use super::profile::{SaveWorkspaceProfileInput, WorkspaceProfile};
use crate::product::permissions::policy::{expand_home, normalize_scope_pattern};
use crate::product::permissions::scope::{PermissionKind, PermissionScope};
use crate::product::security::path_guard::{ensure_under_root, reject_traversal};
use anyhow::{anyhow, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceFile {
    version: u32,
    profiles: Vec<WorkspaceProfile>,
}

pub struct WorkspaceStore {
    path: PathBuf,
}

impl WorkspaceStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list(&self) -> anyhow::Result<Vec<WorkspaceProfile>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let raw = fs::read_to_string(&self.path)?;
        if raw.trim().is_empty() {
            return Ok(vec![]);
        }

        let file: WorkspaceFile = serde_json::from_str(&raw)?;
        Ok(file.profiles)
    }

    pub fn save_profile(
        &self,
        input: SaveWorkspaceProfileInput,
    ) -> anyhow::Result<WorkspaceProfile> {
        let normalized = normalize_input(input)?;
        let mut profiles = self.list()?;
        let now = Utc::now().timestamp_millis();

        if let Some(id) = normalized.id.as_ref() {
            let profile = profiles
                .iter_mut()
                .find(|profile| &profile.id == id)
                .ok_or_else(|| anyhow!("workspace profile not found"))?;

            profile.name = normalized.name;
            profile.root_path = normalized.root_path;
            profile.permission_scopes = normalized.permission_scopes;
            profile.updated_at = now;

            let result = profile.clone();
            self.save_all(&profiles)?;
            return Ok(result);
        }

        let profile = WorkspaceProfile {
            id: Uuid::new_v4().to_string(),
            name: normalized.name,
            root_path: normalized.root_path,
            permission_scopes: normalized.permission_scopes,
            created_at: now,
            updated_at: now,
        };

        profiles.push(profile.clone());
        self.save_all(&profiles)?;

        Ok(profile)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<Vec<WorkspaceProfile>> {
        let mut profiles = self.list()?;
        let before = profiles.len();

        profiles.retain(|profile| profile.id != id);

        if profiles.len() == before {
            bail!("workspace profile not found");
        }

        self.save_all(&profiles)?;
        Ok(profiles)
    }

    fn save_all(&self, profiles: &[WorkspaceProfile]) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = WorkspaceFile {
            version: SCHEMA_VERSION,
            profiles: profiles.to_vec(),
        };

        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&file)?)?;
        fs::rename(tmp, &self.path)?;

        Ok(())
    }
}

fn normalize_input(input: SaveWorkspaceProfileInput) -> anyhow::Result<SaveWorkspaceProfileInput> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        bail!("workspace name is required");
    }

    let root = PathBuf::from(expand_home(input.root_path.trim()));
    reject_traversal(&root)?;

    let canonical_root = root
        .canonicalize()
        .map_err(|err| anyhow!("invalid workspace root: {err}"))?;

    if !canonical_root.is_dir() {
        bail!("workspace root is not a directory");
    }

    let permission_scopes = input
        .permission_scopes
        .into_iter()
        .map(|scope| normalize_workspace_scope(&canonical_root, scope))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(SaveWorkspaceProfileInput {
        id: input.id,
        name,
        root_path: canonical_root.to_string_lossy().to_string(),
        permission_scopes,
    })
}

fn normalize_workspace_scope(
    workspace_root: &Path,
    scope: PermissionScope,
) -> anyhow::Result<PermissionScope> {
    if scope.kind != PermissionKind::Filesystem {
        bail!("workspace permission scopes must be filesystem scopes");
    }

    let normalized_pattern = normalize_scope_pattern(&scope.pattern)?;
    let scope_root = root_from_scope_pattern(&normalized_pattern);

    ensure_under_root(&scope_root, workspace_root)?;

    Ok(PermissionScope {
        pattern: normalized_pattern,
        ..scope
    })
}

fn root_from_scope_pattern(pattern: &str) -> PathBuf {
    let value = pattern
        .strip_suffix("/**")
        .or_else(|| pattern.strip_suffix("/*"))
        .unwrap_or(pattern);

    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::super::profile::SaveWorkspaceProfileInput;
    use super::{normalize_input, root_from_scope_pattern, WorkspaceStore};
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use tempfile::tempdir;

    #[test]
    fn create_update_remove_profile() {
        let dir = tempdir().unwrap();
        let store = WorkspaceStore::new(dir.path().join("workspaces.json"));

        let scope = PermissionScope {
            id: "s1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: format!("{}/**", dir.path().display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        };

        let created = store
            .save_profile(SaveWorkspaceProfileInput {
                id: None,
                name: "demo".to_string(),
                root_path: dir.path().to_string_lossy().to_string(),
                permission_scopes: vec![scope.clone()],
            })
            .unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(created.name, "demo");
        assert_eq!(created.permission_scopes.len(), 1);

        let updated = store
            .save_profile(SaveWorkspaceProfileInput {
                id: Some(created.id.clone()),
                name: "demo2".to_string(),
                root_path: dir.path().to_string_lossy().to_string(),
                permission_scopes: vec![],
            })
            .unwrap();
        assert_eq!(updated.name, "demo2");
        assert!(updated.permission_scopes.is_empty());
        assert!(updated.updated_at >= created.created_at);

        let profiles = store.remove(&created.id).unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn list_returns_empty_when_file_missing() {
        let dir = tempdir().unwrap();
        let store = WorkspaceStore::new(dir.path().join("workspaces.json"));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn rejects_workspace_scope_outside_root() {
        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();

        let scope = PermissionScope {
            id: "s1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: format!("{}/**", outside.path().display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        };

        let result = normalize_input(SaveWorkspaceProfileInput {
            id: None,
            name: "demo".to_string(),
            root_path: root.path().to_string_lossy().to_string(),
            permission_scopes: vec![scope],
        });

        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("escapes") || err.contains("not authorized"),
            "got: {err}",
        );
    }

    #[test]
    fn rejects_non_existing_workspace_root() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");

        let result = normalize_input(SaveWorkspaceProfileInput {
            id: None,
            name: "demo".to_string(),
            root_path: missing.to_string_lossy().to_string(),
            permission_scopes: vec![],
        });

        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_filesystem_scope_kind() {
        let dir = tempdir().unwrap();

        let scope = PermissionScope {
            id: "s1".to_string(),
            kind: PermissionKind::Command,
            pattern: format!("{}/**", dir.path().display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        };

        let result = normalize_input(SaveWorkspaceProfileInput {
            id: None,
            name: "demo".to_string(),
            root_path: dir.path().to_string_lossy().to_string(),
            permission_scopes: vec![scope],
        });

        let err = result.unwrap_err().to_string();
        assert!(err.contains("filesystem"), "got: {err}");
    }

    #[test]
    fn root_from_scope_pattern_strips_glob_suffix() {
        assert_eq!(root_from_scope_pattern("/a/b/**").to_string_lossy(), "/a/b");
        assert_eq!(root_from_scope_pattern("/a/b/*").to_string_lossy(), "/a/b");
        assert_eq!(
            root_from_scope_pattern("/a/b/c").to_string_lossy(),
            "/a/b/c"
        );
    }
}
