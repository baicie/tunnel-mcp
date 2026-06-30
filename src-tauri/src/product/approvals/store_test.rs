#[cfg(test)]
mod tests {
    use super::super::request::{ApprovalStatus, ApprovalTool, NewApprovalRequest};
    use super::super::store::ApprovalStore;
    use sha2::{Digest, Sha256};
    use tempfile::tempdir;

    fn sha256_hex(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn input(ttl_seconds: i64) -> NewApprovalRequest {
        NewApprovalRequest {
            tool: ApprovalTool::FilesWrite,
            target_path: "/tmp/a.txt".to_string(),
            summary: "write file".to_string(),
            diff: None,
            content_sha256: Some(sha256_hex("hello")),
            ttl_seconds,
        }
    }

    #[test]
    fn create_and_approve_request() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        let request = store.create(input(300)).unwrap();
        assert_eq!(request.status, ApprovalStatus::Pending);

        let approved = store.approve(&request.id).unwrap();
        assert_eq!(approved.status, ApprovalStatus::Approved);
        assert!(store
            .get_valid_approved(&request.id, ApprovalTool::FilesWrite, "/tmp/a.txt", None)
            .is_ok());
    }

    #[test]
    fn reject_blocks_valid_approved_lookup() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        let request = store.create(input(300)).unwrap();
        store.reject(&request.id).unwrap();
        assert!(store
            .get_valid_approved(&request.id, ApprovalTool::FilesWrite, "/tmp/a.txt", None)
            .is_err());
    }

    #[test]
    fn expired_request_cannot_be_approved() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        let request = store.create(input(1)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let requests = store.list().unwrap();
        assert_eq!(requests[0].status, ApprovalStatus::Expired);
        assert!(store.approve(&request.id).is_err());
    }

    #[test]
    fn create_rejects_non_positive_ttl() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        assert!(store
            .create(NewApprovalRequest {
                content_sha256: None,
                ..input(-1)
            })
            .is_err());
        assert!(store
            .create(NewApprovalRequest {
                content_sha256: None,
                ..input(0)
            })
            .is_err());
    }

    #[test]
    fn approved_request_expires_before_execution() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));

        let request = store.create(input(1)).unwrap();
        store.approve(&request.id).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1100));

        assert!(store
            .get_valid_approved(&request.id, ApprovalTool::FilesWrite, "/tmp/a.txt", None)
            .is_err());
    }

    #[test]
    fn approval_tool_serializes_as_mcp_tool_name() {
        assert_eq!(
            serde_json::to_string(&ApprovalTool::FilesWrite).unwrap(),
            "\"files.write\""
        );
        assert_eq!(
            serde_json::to_string(&ApprovalTool::FilesPatch).unwrap(),
            "\"files.patch\""
        );
    }
}
