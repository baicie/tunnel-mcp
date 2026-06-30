#[cfg(test)]
mod tests {
    use super::super::request::{ApprovalTool, NewApprovalRequest};
    use super::super::store::ApprovalStore;
    use super::super::write_guard::{sha256_hex, write_file_with_approval};
    use super::super::write_log::WriteLogStore;
    use crate::product::permissions::policy::PermissionPolicy;
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use std::fs;
    use tempfile::tempdir;

    fn policy_for(dir: &std::path::Path) -> PermissionPolicy {
        PermissionPolicy::new(vec![PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: format!("{}/*", dir.display()),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        }])
        .unwrap()
    }

    #[test]
    fn write_requires_approved_request() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let log_dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "").unwrap();
        let policy = policy_for(dir.path());
        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let logs = WriteLogStore::new(log_dir.path().join("write-log.json"));
        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: target.canonicalize().unwrap().to_string_lossy().to_string(),
                summary: "write".to_string(),
                diff: None,
                content_sha256: Some(sha256_hex("hello")),
                ttl_seconds: 300,
            })
            .unwrap();

        assert!(write_file_with_approval(
            &policy,
            &store,
            &logs,
            &request.id,
            target.clone(),
            "hello",
        )
        .is_err());
        store.approve(&request.id).unwrap();
        write_file_with_approval(&policy, &store, &logs, &request.id, target.clone(), "hello")
            .unwrap();
        assert_eq!(fs::read_to_string(target).unwrap(), "hello");
    }

    #[test]
    fn target_path_must_match_approval() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let log_dir = tempdir().unwrap();
        let approved_target = dir.path().join("a.txt");
        let other_target = dir.path().join("b.txt");
        fs::write(&approved_target, "").unwrap();
        fs::write(&other_target, "").unwrap();
        let policy = policy_for(dir.path());
        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let logs = WriteLogStore::new(log_dir.path().join("write-log.json"));
        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: approved_target
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                summary: "write".to_string(),
                diff: None,
                content_sha256: Some(sha256_hex("hello")),
                ttl_seconds: 300,
            })
            .unwrap();
        store.approve(&request.id).unwrap();

        assert!(write_file_with_approval(
            &policy,
            &store,
            &logs,
            &request.id,
            other_target,
            "hello",
        )
        .is_err());
    }

    #[test]
    fn content_hash_must_match_approval() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let log_dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "").unwrap();

        let policy = policy_for(dir.path());

        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let logs = WriteLogStore::new(log_dir.path().join("write-log.json"));

        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: target.canonicalize().unwrap().to_string_lossy().to_string(),
                summary: "write".to_string(),
                diff: None,
                content_sha256: Some(sha256_hex("hello")),
                ttl_seconds: 300,
            })
            .unwrap();

        store.approve(&request.id).unwrap();

        assert!(write_file_with_approval(
            &policy,
            &store,
            &logs,
            &request.id,
            target,
            "different content",
        )
        .is_err());
    }

    #[test]
    fn write_log_is_appended_for_succeeded_writes() {
        let dir = tempdir().unwrap();
        let approval_dir = tempdir().unwrap();
        let log_dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "").unwrap();

        let policy = policy_for(dir.path());
        let store = ApprovalStore::new(approval_dir.path().join("approvals.json"));
        let logs = WriteLogStore::new(log_dir.path().join("write-log.json"));

        let request = store
            .create(NewApprovalRequest {
                tool: ApprovalTool::FilesWrite,
                target_path: target.canonicalize().unwrap().to_string_lossy().to_string(),
                summary: "write".to_string(),
                diff: None,
                content_sha256: Some(sha256_hex("hello")),
                ttl_seconds: 300,
            })
            .unwrap();
        store.approve(&request.id).unwrap();

        write_file_with_approval(&policy, &store, &logs, &request.id, target.clone(), "hello")
            .unwrap();

        let entries = logs.list().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].approval_id, request.id);
        assert_eq!(
            entries[0].status,
            super::super::write_log::WriteLogStatus::Succeeded
        );
    }
}
