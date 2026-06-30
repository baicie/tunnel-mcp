#[cfg(test)]
mod tests {
    use crate::product::mcp::resources::{
        list_authorized_resources, list_files, read_file, AllowRootsReadPolicy,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn list_files_allows_authorized_root() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();
        let entries = list_files(dir.path(), &policy).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "a.txt");
    }

    #[test]
    fn read_file_allows_authorized_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();
        let result = read_file(&file, &policy).unwrap();
        assert_eq!(result.content, "hello");
    }

    #[test]
    fn read_file_rejects_unauthorized_file() {
        let allowed = tempdir().unwrap();
        let denied = tempdir().unwrap();
        let file = denied.path().join("secret.txt");
        fs::write(&file, "secret").unwrap();
        let policy = AllowRootsReadPolicy::new(vec![allowed.path().to_path_buf()]).unwrap();
        assert!(read_file(&file, &policy).is_err());
    }

    #[test]
    fn empty_policy_denies_every_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("secret.txt");
        fs::write(&file, "secret").unwrap();

        let policy = AllowRootsReadPolicy::new(vec![]).unwrap();

        assert!(read_file(&file, &policy).is_err());
    }

    #[test]
    fn read_file_rejects_large_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("large.txt");
        fs::write(&file, vec![b'a'; 1024 * 1024 + 1]).unwrap();

        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();
        let err = read_file(&file, &policy).unwrap_err().to_string();

        assert!(err.contains("too large"));
    }

    #[test]
    fn list_authorized_resources_is_empty_by_default() {
        let policy = AllowRootsReadPolicy::new(vec![]).unwrap();
        let resources = list_authorized_resources(&policy);

        assert!(resources.is_empty());
    }
}
