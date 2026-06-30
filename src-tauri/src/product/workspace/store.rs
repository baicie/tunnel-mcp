use super::profile::{SaveWorkspaceProfileInput, WorkspaceProfile};
use anyhow::anyhow;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceFile {
    version: u32,
    profiles: Vec<WorkspaceProfile>,
}

#[allow(dead_code)]
pub struct WorkspaceStore {
    path: PathBuf,
}

impl WorkspaceStore {
    #[allow(dead_code)]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[allow(dead_code)]
    pub fn list(&self) -> anyhow::Result<Vec<WorkspaceProfile>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let raw = fs::read_to_string(&self.path)?;
        let file: WorkspaceFile = serde_json::from_str(&raw)?;
        Ok(file.profiles)
    }

    #[allow(dead_code)]
    pub fn save_profile(
        &self,
        input: SaveWorkspaceProfileInput,
    ) -> anyhow::Result<WorkspaceProfile> {
        let mut profiles = self.list()?;
        let now = Utc::now().timestamp_millis();

        if let Some(id) = input.id.as_ref() {
            let profile = profiles
                .iter_mut()
                .find(|profile| &profile.id == id)
                .ok_or_else(|| anyhow!("workspace profile not found"))?;
            profile.name = input.name;
            profile.root_path = input.root_path;
            profile.permission_scopes = input.permission_scopes;
            profile.updated_at = now;
            let result = profile.clone();
            self.save_all(&profiles)?;
            return Ok(result);
        }

        let profile = WorkspaceProfile {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            root_path: input.root_path,
            permission_scopes: input.permission_scopes,
            created_at: now,
            updated_at: now,
        };
        profiles.push(profile.clone());
        self.save_all(&profiles)?;
        Ok(profile)
    }

    #[allow(dead_code)]
    pub fn remove(&self, id: &str) -> anyhow::Result<Vec<WorkspaceProfile>> {
        let profiles = self
            .list()?
            .into_iter()
            .filter(|profile| profile.id != id)
            .collect::<Vec<_>>();
        self.save_all(&profiles)?;
        Ok(profiles)
    }

    fn save_all(&self, profiles: &[WorkspaceProfile]) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = WorkspaceFile {
            version: 1,
            profiles: profiles.to_vec(),
        };
        fs::write(&self.path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::profile::SaveWorkspaceProfileInput;
    use super::WorkspaceStore;
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use tempfile::tempdir;

    #[test]
    fn create_update_remove_profile() {
        let dir = tempdir().unwrap();
        let store = WorkspaceStore::new(dir.path().join("workspaces.json"));

        let scope = PermissionScope {
            id: "s1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: "/tmp/demo/**".to_string(),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        };

        let created = store
            .save_profile(SaveWorkspaceProfileInput {
                id: None,
                name: "demo".to_string(),
                root_path: "/tmp/demo".to_string(),
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
                root_path: "/tmp/demo2".to_string(),
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
}
