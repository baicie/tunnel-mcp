use super::request::{
    ApprovalRequest, ApprovalSource, ApprovalStatus, ApprovalTool, NewApprovalRequest,
};
use anyhow::{anyhow, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct ApprovalFile {
    version: u32,
    requests: Vec<ApprovalRequest>,
}

pub struct ApprovalStore {
    path: PathBuf,
}

impl ApprovalStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list(&self) -> anyhow::Result<Vec<ApprovalRequest>> {
        let mut requests = self.load()?;
        expire_requests(&mut requests);
        self.save(&requests)?;
        Ok(requests)
    }

    pub fn create(&self, input: NewApprovalRequest) -> anyhow::Result<ApprovalRequest> {
        if input.ttl_seconds <= 0 {
            bail!("approval ttl must be positive");
        }

        if input.target_path.trim().is_empty() {
            bail!("approval target path is required");
        }

        let mut requests = self.load()?;
        expire_requests(&mut requests);

        let now = Utc::now().timestamp_millis();
        let request = ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            source: ApprovalSource::Mcp,
            tool: input.tool,
            target_path: input.target_path,
            summary: input.summary,
            diff: input.diff,
            content_sha256: input.content_sha256,
            created_at: now,
            expires_at: now + input.ttl_seconds * 1000,
            status: ApprovalStatus::Pending,
        };

        requests.push(request.clone());
        self.save(&requests)?;

        Ok(request)
    }

    pub fn approve(&self, id: &str) -> anyhow::Result<ApprovalRequest> {
        self.transition(id, ApprovalStatus::Approved)
    }

    pub fn reject(&self, id: &str) -> anyhow::Result<ApprovalRequest> {
        self.transition(id, ApprovalStatus::Rejected)
    }

    pub fn get_valid_approved(
        &self,
        id: &str,
        expected_tool: ApprovalTool,
        expected_target_path: &str,
        expected_content_sha256: Option<&str>,
    ) -> anyhow::Result<ApprovalRequest> {
        let mut requests = self.load()?;
        expire_requests(&mut requests);
        self.save(&requests)?;

        let request = requests
            .into_iter()
            .find(|request| request.id == id)
            .ok_or_else(|| anyhow!("approval request not found"))?;

        if request.status != ApprovalStatus::Approved {
            bail!("approval request is not approved");
        }

        if request.expires_at <= Utc::now().timestamp_millis() {
            bail!("approval request expired");
        }

        if request.tool != expected_tool {
            bail!("approval tool mismatch");
        }

        if request.target_path != expected_target_path {
            bail!("approval target path mismatch");
        }

        if let Some(expected_hash) = expected_content_sha256 {
            if request.content_sha256.as_deref() != Some(expected_hash) {
                bail!("approval content hash mismatch");
            }
        }

        Ok(request)
    }

    fn transition(&self, id: &str, status: ApprovalStatus) -> anyhow::Result<ApprovalRequest> {
        let mut requests = self.load()?;
        expire_requests(&mut requests);

        let request = requests
            .iter_mut()
            .find(|request| request.id == id)
            .ok_or_else(|| anyhow!("approval request not found"))?;

        if request.status != ApprovalStatus::Pending {
            bail!("approval request is not pending");
        }

        if request.expires_at <= Utc::now().timestamp_millis() {
            request.status = ApprovalStatus::Expired;
            self.save(&requests)?;
            bail!("approval request expired");
        }

        request.status = status;
        let result = request.clone();
        self.save(&requests)?;

        Ok(result)
    }

    fn load(&self) -> anyhow::Result<Vec<ApprovalRequest>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let raw = fs::read_to_string(&self.path)?;
        if raw.trim().is_empty() {
            return Ok(vec![]);
        }

        let file: ApprovalFile = serde_json::from_str(&raw)?;
        Ok(file.requests)
    }

    fn save(&self, requests: &[ApprovalRequest]) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = ApprovalFile {
            version: SCHEMA_VERSION,
            requests: requests.to_vec(),
        };

        fs::write(&self.path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }
}

fn expire_requests(requests: &mut [ApprovalRequest]) {
    let now = Utc::now().timestamp_millis();

    for request in requests {
        if matches!(
            request.status,
            ApprovalStatus::Pending | ApprovalStatus::Approved
        ) && request.expires_at <= now
        {
            request.status = ApprovalStatus::Expired;
        }
    }
}
