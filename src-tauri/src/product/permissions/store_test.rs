#[cfg(test)]
mod tests {
    use super::super::scope::{NewPermissionScope, PermissionAccess, PermissionKind};
    use super::super::store::PermissionStore;
    use tempfile::tempdir;

    #[test]
    fn add_and_remove_scope() {
        let dir = tempdir().unwrap();
        let store = PermissionStore::new(dir.path().join("permissions.json"));
        let scope = store
            .add(NewPermissionScope {
                kind: PermissionKind::Filesystem,
                pattern: "~/Documents/**".to_string(),
                access: PermissionAccess::Read,
                require_approval: false,
            })
            .unwrap();

        assert_eq!(store.list().unwrap().len(), 1);
        let scopes = store.remove(&scope.id).unwrap();
        assert!(scopes.is_empty());
    }

    #[test]
    fn list_returns_empty_when_file_missing() {
        let dir = tempdir().unwrap();
        let store = PermissionStore::new(dir.path().join("missing.json"));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn add_persists_across_instances() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("permissions.json");

        let added = PermissionStore::new(path.clone())
            .add(NewPermissionScope {
                kind: PermissionKind::Filesystem,
                pattern: "~/project/**".to_string(),
                access: PermissionAccess::Readwrite,
                require_approval: true,
            })
            .unwrap();

        let reloaded = PermissionStore::new(path).list().unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].id, added.id);
        assert!(reloaded[0].require_approval);
    }

    #[test]
    fn rejects_empty_pattern() {
        let dir = tempdir().unwrap();
        let store = PermissionStore::new(dir.path().join("permissions.json"));

        let result = store.add(NewPermissionScope {
            kind: PermissionKind::Filesystem,
            pattern: "   ".to_string(),
            access: PermissionAccess::Read,
            require_approval: false,
        });

        assert!(result.is_err());
    }

    #[test]
    fn rejects_command_scope_in_phase_4() {
        let dir = tempdir().unwrap();
        let store = PermissionStore::new(dir.path().join("permissions.json"));

        let result = store.add(NewPermissionScope {
            kind: PermissionKind::Command,
            pattern: "command/run".to_string(),
            access: PermissionAccess::Read,
            require_approval: true,
        });

        assert!(result.is_err());
    }
}
