use super::policy::normalize_scope_pattern;
use super::scope::{NewPermissionScope, PermissionKind, PermissionScope};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct PermissionFile {
    version: u32,
    scopes: Vec<PermissionScope>,
}

pub struct PermissionStore {
    path: PathBuf,
}

impl PermissionStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list(&self) -> anyhow::Result<Vec<PermissionScope>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let raw = fs::read_to_string(&self.path)?;
        if raw.trim().is_empty() {
            return Ok(vec![]);
        }

        let file: PermissionFile = serde_json::from_str(&raw)?;
        Ok(file.scopes)
    }

    pub fn add(&self, input: NewPermissionScope) -> anyhow::Result<PermissionScope> {
        if input.kind != PermissionKind::Filesystem {
            return Err(anyhow!(
                "only filesystem permission scopes are supported in phase 4"
            ));
        }

        let normalized_pattern = normalize_scope_pattern(&input.pattern)?;

        let mut scopes = self.list()?;

        let scope = PermissionScope {
            id: Uuid::new_v4().to_string(),
            kind: input.kind,
            pattern: normalized_pattern,
            access: input.access,
            require_approval: input.require_approval,
        };

        scopes.push(scope.clone());
        self.save(scopes)?;

        Ok(scope)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<Vec<PermissionScope>> {
        let scopes = self
            .list()?
            .into_iter()
            .filter(|scope| scope.id != id)
            .collect::<Vec<_>>();

        self.save(scopes.clone())?;
        Ok(scopes)
    }

    fn save(&self, scopes: Vec<PermissionScope>) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = PermissionFile {
            version: SCHEMA_VERSION,
            scopes,
        };

        fs::write(&self.path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }
}
