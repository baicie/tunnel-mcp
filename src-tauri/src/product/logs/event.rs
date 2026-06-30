use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEvent {
    pub id: String,
    pub request_id: Option<String>,
    pub r#type: String,
    pub level: LogLevel,
    pub message: String,
    pub metadata: Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLogsInput {
    pub r#type: Option<String>,
    pub request_id: Option<String>,
    pub limit: Option<usize>,
}
