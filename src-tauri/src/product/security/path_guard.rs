use anyhow::{anyhow, bail};
use std::path::{Component, Path, PathBuf};

pub fn reject_traversal(path: &Path) -> anyhow::Result<()> {
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            bail!("path traversal is not allowed");
        }
    }
    Ok(())
}

pub fn canonicalize_existing_or_parent(path: &Path) -> anyhow::Result<PathBuf> {
    reject_traversal(path)?;

    if path.exists() {
        return path.canonicalize().map_err(Into::into);
    }

    let parent = path.parent().ok_or_else(|| anyhow!("path has no parent"))?;
    reject_traversal(parent)?;

    let canonical_parent = parent.canonicalize()?;
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("path has no file name"))?;

    Ok(canonical_parent.join(name))
}

#[allow(dead_code)]
pub fn ensure_under_root(path: &Path, root: &Path) -> anyhow::Result<PathBuf> {
    let path = canonicalize_existing_or_parent(path)?;
    let root = root.canonicalize()?;
    if !path.starts_with(&root) {
        bail!("path escapes authorized root");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_existing_or_parent, ensure_under_root, reject_traversal};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_parent_dir_traversal() {
        assert!(reject_traversal(std::path::Path::new("../secret")).is_err());
        assert!(reject_traversal(std::path::Path::new("safe/file.txt")).is_ok());
    }

    #[test]
    fn canonicalizes_non_existing_file_by_parent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new.txt");
        let canonical = canonicalize_existing_or_parent(&path).unwrap();
        assert!(canonical.ends_with("new.txt"));
    }

    #[test]
    fn ensures_path_under_root() {
        let root = tempdir().unwrap();
        let file = root.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        assert!(ensure_under_root(&file, root.path()).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn ensure_under_root_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let outside_file = outside.path().join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();

        symlink(&outside_file, root.path().join("link.txt")).unwrap();
        assert!(ensure_under_root(&root.path().join("link.txt"), root.path()).is_err());
    }
}
