#[cfg(test)]
mod tests {
    use super::super::request::{ApprovalStatus, ApprovalTool, NewApprovalRequest};
    use super::super::store::ApprovalStore;
    use tempfile::tempdir;

    fn input(ttl_seconds: i64) -> NewApprovalRequest {
        NewApprovalRequest {
            tool: ApprovalTool::FilesWrite,
            target_path: "/tmp/a.txt".to_string(),
            summary: "write file".to_string(),
            diff: None,
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
        assert!(store.get_valid_approved(&request.id).is_ok());
    }

    #[test]
    fn reject_blocks_valid_approved_lookup() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        let request = store.create(input(300)).unwrap();
        store.reject(&request.id).unwrap();
        assert!(store.get_valid_approved(&request.id).is_err());
    }

    #[test]
    fn expired_request_cannot_be_approved() {
        let dir = tempdir().unwrap();
        let store = ApprovalStore::new(dir.path().join("approvals.json"));
        let request = store.create(input(-1)).unwrap();
        let requests = store.list().unwrap();
        assert_eq!(requests[0].status, ApprovalStatus::Expired);
        assert!(store.approve(&request.id).is_err());
    }
}
