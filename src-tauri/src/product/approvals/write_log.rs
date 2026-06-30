use super::request::ApprovalTool;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WriteLogStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WriteLogEntry {
    pub id: String,
    pub approval_id: String,
    pub tool: ApprovalTool,
    pub target_path: String,
    pub content_sha256: Option<String>,
    pub status: WriteLogStatus,
    pub error: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteLogFile {
    version: u32,
    entries: Vec<WriteLogEntry>,
}

pub struct WriteLogStore {
    path: PathBuf,
}

impl WriteLogStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(&self, mut entry: WriteLogEntry) -> anyhow::Result<WriteLogEntry> {
        let mut entries = self.list()?;

        if entry.id.is_empty() {
            entry.id = Uuid::new_v4().to_string();
        }

        if entry.created_at == 0 {
            entry.created_at = Utc::now().timestamp_millis();
        }

        entries.push(entry.clone());
        self.save(&entries)?;

        Ok(entry)
    }

    pub fn list(&self) -> anyhow::Result<Vec<WriteLogEntry>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let raw = fs::read_to_string(&self.path)?;
        if raw.trim().is_empty() {
            return Ok(vec![]);
        }

        let file: WriteLogFile = serde_json::from_str(&raw)?;
        Ok(file.entries)
    }

    fn save(&self, entries: &[WriteLogEntry]) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = WriteLogFile {
            version: 1,
            entries: entries.to_vec(),
        };

        fs::write(&self.path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }
}
