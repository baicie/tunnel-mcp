use super::request::{ApprovalRequest, ApprovalStatus, NewApprovalRequest};
use anyhow::{anyhow, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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

    #[allow(dead_code)]
    pub fn create(&self, input: NewApprovalRequest) -> anyhow::Result<ApprovalRequest> {
        let mut requests = self.load()?;
        let now = Utc::now().timestamp_millis();
        let request = ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            source: "mcp".to_string(),
            tool: input.tool,
            target_path: input.target_path,
            summary: input.summary,
            diff: input.diff,
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

    #[allow(dead_code)]
    pub fn get_valid_approved(&self, id: &str) -> anyhow::Result<ApprovalRequest> {
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
        let file: ApprovalFile = serde_json::from_str(&raw)?;
        Ok(file.requests)
    }

    fn save(&self, requests: &[ApprovalRequest]) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = ApprovalFile {
            version: 1,
            requests: requests.to_vec(),
        };
        fs::write(&self.path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }
}

fn expire_requests(requests: &mut [ApprovalRequest]) {
    let now = Utc::now().timestamp_millis();
    for request in requests {
        if request.status == ApprovalStatus::Pending && request.expires_at <= now {
            request.status = ApprovalStatus::Expired;
        }
    }
}
