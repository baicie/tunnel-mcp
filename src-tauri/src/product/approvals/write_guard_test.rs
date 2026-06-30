#[cfg(test)]
mod tests {
    use super::super::request::{ApprovalTool, NewApprovalRequest};
    use super::super::store::ApprovalStore;
    use super::super::write_guard::write_file_with_approval;
    use crate::product::permissions::policy::PermissionPolicy;
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn write_requires_approved_request() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "").unwrap();
        let policy = PermissionPolicy::new(vec![PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: format!("{}/*", dir.path().display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        }])
        .unwrap();
        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: target.to_string_lossy().to_string(),
                summary: "write".to_string(),
                diff: None,
                ttl_seconds: 300,
            })
            .unwrap();

        assert!(
            write_file_with_approval(&policy, &store, &request.id, target.clone(), "hello")
                .is_err()
        );
        store.approve(&request.id).unwrap();
        write_file_with_approval(&policy, &store, &request.id, target.clone(), "hello").unwrap();
        assert_eq!(fs::read_to_string(target).unwrap(), "hello");
    }

    #[test]
    fn target_path_must_match_approval() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let approved_target = dir.path().join("a.txt");
        let other_target = dir.path().join("b.txt");
        fs::write(&approved_target, "").unwrap();
        fs::write(&other_target, "").unwrap();
        let policy = PermissionPolicy::new(vec![PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: format!("{}/*", dir.path().display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        }])
        .unwrap();
        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: approved_target.to_string_lossy().to_string(),
                summary: "write".to_string(),
                diff: None,
                ttl_seconds: 300,
            })
            .unwrap();
        store.approve(&request.id).unwrap();

        assert!(
            write_file_with_approval(&policy, &store, &request.id, other_target, "hello").is_err()
        );
    }
}
