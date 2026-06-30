#[cfg(test)]
mod tests {
    use super::super::policy::{PermissionPolicy, DENY_REASON_SENSITIVE_PATH};
    use super::super::scope::{PermissionAccess, PermissionKind, PermissionScope};
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
    fn allows_read_inside_authorized_scope() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/*", dir.path().display()),
            PermissionAccess::Read,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Read);
        assert!(decision.allowed, "reason={:?}", decision.reason);
        assert!(!decision.require_approval);
    }

    #[test]
    fn write_requires_approval_even_if_scope_says_readwrite() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/*", dir.path().display()),
            PermissionAccess::Readwrite,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Write);
        assert!(decision.allowed);
        assert!(decision.require_approval);
    }

    #[test]
    fn rejects_path_outside_scope() {
        let allowed = tempdir().unwrap();
        let denied = tempdir().unwrap();
        let file = denied.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/*", allowed.path().display()),
            PermissionAccess::Read,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Read);
        assert!(!decision.allowed);
    }

    #[test]
    fn rejects_sensitive_path_even_if_scope_matches() {
        let dir = tempdir().unwrap();
        let ssh = dir.path().join(".ssh");
        fs::create_dir_all(&ssh).unwrap();
        let file = ssh.join("id_rsa");
        fs::write(&file, "secret").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/**", dir.path().display()),
            PermissionAccess::Readwrite,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Read);
        assert!(!decision.allowed);
        assert_eq!(decision.reason, DENY_REASON_SENSITIVE_PATH);
    }

    #[test]
    fn rejects_dotenv_even_if_scope_matches() {
        let dir = tempdir().unwrap();
        let env = dir.path().join(".env");
        fs::write(&env, "TOKEN=secret").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/**", dir.path().display()),
            PermissionAccess::Readwrite,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&env, PermissionAccess::Read);
        assert!(!decision.allowed);
        assert_eq!(decision.reason, DENY_REASON_SENSITIVE_PATH);
    }

    #[test]
    fn allowlist_with_no_scopes_denies_everything() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "ok").unwrap();
        let policy = PermissionPolicy::new(vec![]).unwrap();
        assert!(!policy.check_path(&file, PermissionAccess::Read).allowed);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let allowed = tempdir().unwrap();
        let denied = tempdir().unwrap();
        let secret = denied.path().join("secret.txt");
        fs::write(&secret, "secret").unwrap();
        let link = allowed.path().join("link.txt");
        symlink(&secret, &link).unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/*", allowed.path().display()),
            PermissionAccess::Read,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&link, PermissionAccess::Read);
        assert!(!decision.allowed);
    }

    #[test]
    fn rejects_sensitive_directory_itself_even_if_scope_matches() {
        let dir = tempdir().unwrap();
        let ssh = dir.path().join(".ssh");
        fs::create_dir_all(&ssh).unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/**", dir.path().display()),
            PermissionAccess::Readwrite,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&ssh, PermissionAccess::Read);
        assert!(!decision.allowed);
        assert_eq!(decision.reason, DENY_REASON_SENSITIVE_PATH);
    }

    #[test]
    fn plain_directory_scope_authorizes_children() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("child.txt");
        fs::write(&file, "hello").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            dir.path().display().to_string(),
            PermissionAccess::Read,
            false,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Read);
        assert!(decision.allowed);
        assert!(!decision.require_approval);
    }

    #[test]
    fn read_with_require_approval_is_not_preapproved() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let policy = PermissionPolicy::new(vec![scope(
            format!("{}/*", dir.path().display()),
            PermissionAccess::Read,
            true,
        )])
        .unwrap();
        let decision = policy.check_path(&file, PermissionAccess::Read);
        assert!(decision.allowed);
        assert!(decision.require_approval);
    }
}
