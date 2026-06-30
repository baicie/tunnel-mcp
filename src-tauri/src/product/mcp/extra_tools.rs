#![allow(dead_code)]

use super::resources::ReadPolicy;
use anyhow::{anyhow, bail};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_SEARCH_LIMIT: usize = 100;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_PREVIEW_CHARS: usize = 240;
const MAX_PACKAGE_JSON_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub preview: String,
}

pub fn files_search(
    root: &Path,
    query: &str,
    limit: usize,
    policy: &dyn ReadPolicy,
) -> anyhow::Result<Vec<SearchResult>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("search query is required");
    }

    let limit = limit.clamp(1, MAX_SEARCH_LIMIT);
    let root = canonical_authorized_dir(root, policy)?;

    let mut results = Vec::new();
    search_dir(&root, query, limit, policy, &mut results)?;

    Ok(results)
}

fn search_dir(
    dir: &Path,
    query: &str,
    limit: usize,
    policy: &dyn ReadPolicy,
    results: &mut Vec<SearchResult>,
) -> anyhow::Result<()> {
    if results.len() >= limit {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if should_skip_path(&path)? {
            continue;
        }

        if !policy.can_read(&path) {
            continue;
        }

        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            search_dir(&path, query, limit, policy, results)?;
        } else if metadata.is_file() {
            search_file(&path, query, limit, results)?;
        }

        if results.len() >= limit {
            break;
        }
    }

    Ok(())
}

fn search_file(
    path: &Path,
    query: &str,
    limit: usize,
    results: &mut Vec<SearchResult>,
) -> anyhow::Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_BYTES {
        return Ok(());
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(path)?.read_to_end(&mut bytes)?;

    let Ok(content) = String::from_utf8(bytes) else {
        return Ok(());
    };

    for (index, line) in content.lines().enumerate() {
        if line.contains(query) {
            results.push(SearchResult {
                path: path.to_string_lossy().to_string(),
                line: index + 1,
                preview: truncate_preview(line.trim()),
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    Ok(())
}

/// Read-only `git status --short`. Must never mutate the working tree.
/// Disables optional locks, external diff, pager, and LFS filters to
/// prevent any side effects from running inside an authorized root.
pub fn git_status_readonly(root: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<String> {
    let root = canonical_authorized_dir(root, policy)?;

    let output = git_base_command(&root)
        .arg("status")
        .arg("--short")
        .output()?;

    if !output.status.success() {
        bail!("git status failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Read-only `git diff`. Uses `--` separator and disables optional locks,
/// external diff, pager, and color to avoid any working-tree mutation or
/// unexpected output formatting.
pub fn git_diff_readonly(root: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<String> {
    let root = canonical_authorized_dir(root, policy)?;

    let output = git_base_command(&root)
        .arg("diff")
        .arg("--no-ext-diff")
        .arg("--no-color")
        .arg("--")
        .output()?;

    if !output.status.success() {
        bail!("git diff failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn inspect_package_json(
    root: &Path,
    policy: &dyn ReadPolicy,
) -> anyhow::Result<serde_json::Value> {
    let root = canonical_authorized_dir(root, policy)?;
    let path = root.join("package.json");

    if !policy.can_read(&path) {
        bail!("permission denied");
    }

    let metadata = fs::metadata(&path).map_err(|_| anyhow!("package.json not found"))?;

    if !metadata.is_file() {
        bail!("package.json is not a file");
    }

    if metadata.len() > MAX_PACKAGE_JSON_BYTES {
        bail!("package.json is too large");
    }

    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn canonical_authorized_dir(root: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<PathBuf> {
    crate::product::security::path_guard::reject_traversal(root)?;

    let canonical = root.canonicalize()?;
    if !canonical.is_dir() {
        bail!("root is not a directory");
    }

    if !policy.can_read(&canonical) {
        bail!("permission denied");
    }

    Ok(canonical)
}

/// Returns `true` if the path should be skipped during traversal:
/// symlinks (to avoid following out-of-root links), and well-known
/// directories that are large or contain machine-generated content.
fn should_skip_path(path: &Path) -> anyhow::Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }

    let name = path
        .file_name()
        .map(|v| v.to_string_lossy())
        .unwrap_or_default();

    Ok(matches!(
        name.as_ref(),
        ".git" | "node_modules" | "target" | "dist" | ".next"
    ))
}

/// Builds a `Command` preset with environment variables that disable
/// any optional lock, external diff, pager, or LFS filter to prevent
/// git from modifying the working tree while running inside an
/// authorized root.
fn git_base_command(root: &Path) -> Command {
    let mut command = Command::new("git");

    command.current_dir(root).env("GIT_OPTIONAL_LOCKS", "0");

    command
        .env("GIT_EXTERNAL_DIFF", "")
        .arg("-c")
        .arg("core.pager=cat")
        .arg("-c")
        .arg("diff.external=")
        .arg("-c")
        .arg("filter.lfs.smudge=")
        .arg("-c")
        .arg("filter.lfs.process=");

    command
}

fn truncate_preview(value: &str) -> String {
    value.chars().take(MAX_PREVIEW_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::{files_search, git_diff_readonly, git_status_readonly, inspect_package_json};
    use crate::product::mcp::resources::ReadPolicy;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[derive(Debug, Clone)]
    struct TestReadPolicy {
        root: PathBuf,
    }

    impl ReadPolicy for TestReadPolicy {
        fn can_read(&self, path: &std::path::Path) -> bool {
            path.canonicalize()
                .map(|p| p.starts_with(&self.root))
                .unwrap_or(false)
        }

        fn authorized_roots(&self) -> Vec<PathBuf> {
            vec![self.root.clone()]
        }
    }

    #[test]
    fn files_search_rejects_empty_query() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = TestReadPolicy { root };

        assert!(files_search(dir.path(), "", 10, &policy).is_err());
        assert!(files_search(dir.path(), "   ", 10, &policy).is_err());
    }

    #[test]
    fn files_search_rejects_unauthorized_root() {
        let allowed = tempdir().unwrap();
        let outside = tempdir().unwrap();

        let policy = TestReadPolicy {
            root: allowed.path().canonicalize().unwrap(),
        };

        assert!(files_search(outside.path(), "needle", 10, &policy).is_err());
    }

    #[test]
    fn package_json_requires_authorized_root() {
        let allowed = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("package.json"), r#"{"name":"x"}"#).unwrap();

        let policy = TestReadPolicy {
            root: allowed.path().canonicalize().unwrap(),
        };

        assert!(inspect_package_json(outside.path(), &policy).is_err());
    }

    #[test]
    fn git_status_readonly_requires_authorized_root() {
        let allowed = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join(".git"), "").ok();

        let policy = TestReadPolicy {
            root: allowed.path().canonicalize().unwrap(),
        };

        assert!(git_status_readonly(outside.path(), &policy).is_err());
    }

    #[test]
    fn git_diff_readonly_requires_authorized_root() {
        let allowed = tempdir().unwrap();
        let outside = tempdir().unwrap();

        let policy = TestReadPolicy {
            root: allowed.path().canonicalize().unwrap(),
        };

        assert!(git_diff_readonly(outside.path(), &policy).is_err());
    }

    #[test]
    fn files_search_respects_limit() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = TestReadPolicy { root };

        for n in 0..5 {
            fs::write(dir.path().join(format!("a{n}.txt")), "needle").unwrap();
        }

        let results = files_search(dir.path(), "needle", 3, &policy).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn files_search_skips_node_modules() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = TestReadPolicy { root };

        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        fs::write(dir.path().join("node_modules/secret.txt"), "needle").unwrap();

        let results = files_search(dir.path(), "needle", 10, &policy).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn inspect_package_json_reads_json() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = TestReadPolicy { root };

        fs::write(
            dir.path().join("package.json"),
            r#"{"name":"demo","scripts":{"test":"echo test"}}"#,
        )
        .unwrap();

        let pkg = inspect_package_json(dir.path(), &policy).unwrap();
        assert_eq!(pkg["name"], "demo");
    }

    #[test]
    fn inspect_package_json_missing_errors() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = TestReadPolicy { root };

        assert!(inspect_package_json(dir.path(), &policy).is_err());
    }
}
