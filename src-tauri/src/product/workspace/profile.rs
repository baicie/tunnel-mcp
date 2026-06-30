use crate::product::permissions::scope::PermissionScope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProfile {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub permission_scopes: Vec<PermissionScope>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWorkspaceProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub root_path: String,
    pub permission_scopes: Vec<PermissionScope>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceProfilesInput {
    #[serde(default)]
    pub root_path: Option<String>,
}
