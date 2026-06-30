#[cfg(test)]
mod tests {
    use crate::product::mcp::resources::ReadPolicy;
    use crate::product::permissions::read_policy::PermissionReadPolicy;
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use std::fs;
    use tempfile::tempdir;

    fn scope(pattern: String, access: PermissionAccess, require_approval: bool) -> PermissionScope {
        PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern,
            access,
            require_approval,
        }
    }

    #[test]
    fn denies_everything_with_empty_scopes() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();

        let policy = PermissionReadPolicy::new(vec![]).unwrap();

        assert!(!policy.can_read(&file));
        assert!(policy.authorized_roots().is_empty());
    }

    #[test]
    fn allows_preapproved_read_scope() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();

        let policy = PermissionReadPolicy::new(vec![scope(
            dir.path().display().to_string(),
            PermissionAccess::Read,
            false,
        )])
        .unwrap();

        assert!(policy.can_read(&file));
    }

    #[test]
    fn denies_read_scope_that_requires_approval() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();

        let policy = PermissionReadPolicy::new(vec![scope(
            dir.path().display().to_string(),
            PermissionAccess::Read,
            true,
        )])
        .unwrap();

        assert!(!policy.can_read(&file));
    }
}
