use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApprovalTool {
    FilesWrite,
    FilesPatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    pub id: String,
    pub source: String,
    pub tool: ApprovalTool,
    pub target_path: String,
    pub summary: String,
    pub diff: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone)]
pub struct NewApprovalRequest {
    pub tool: ApprovalTool,
    pub target_path: String,
    pub summary: String,
    pub diff: Option<String>,
    pub ttl_seconds: i64,
}
