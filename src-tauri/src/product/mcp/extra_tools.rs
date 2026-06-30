#![allow(dead_code)]

use anyhow::{anyhow, bail};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub preview: String,
}

pub fn files_search(root: &Path, query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    search_dir(root, query, limit, &mut results)?;
    Ok(results)
}

fn search_dir(
    dir: &Path,
    query: &str,
    limit: usize,
    results: &mut Vec<SearchResult>,
) -> anyhow::Result<()> {
    if results.len() >= limit {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            search_dir(&path, query, limit, results)?;
        } else if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                for (index, line) in content.lines().enumerate() {
                    if line.contains(query) {
                        results.push(SearchResult {
                            path: path.to_string_lossy().to_string(),
                            line: index + 1,
                            preview: line.trim().to_string(),
                        });
                        if results.len() >= limit {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Read-only `git status --short`. Must never mutate the working tree.
pub fn git_status_readonly(root: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("status")
        .arg("--short")
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        bail!("git status failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Read-only `git diff`. Uses `--` separator to avoid being misparsed
/// as `git diff path/to/file`.
pub fn git_diff_readonly(root: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--")
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        bail!("git diff failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn inspect_package_json(root: &Path) -> anyhow::Result<serde_json::Value> {
    let path = root.join("package.json");
    if !path.exists() {
        return Err(anyhow!("package.json not found"));
    }
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[cfg(test)]
mod tests {
    use super::{files_search, inspect_package_json};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn files_search_finds_lines() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello\nneedle\n").unwrap();
        let results = files_search(dir.path(), "needle", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, 2);
        assert_eq!(results[0].preview, "needle");
    }

    #[test]
    fn files_search_respects_limit() {
        let dir = tempdir().unwrap();
        for n in 0..5 {
            fs::write(dir.path().join(format!("a{n}.txt")), "needle").unwrap();
        }
        let results = files_search(dir.path(), "needle", 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn inspect_package_json_reads_json() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), r#"{"name":"demo"}"#).unwrap();
        let pkg = inspect_package_json(dir.path()).unwrap();
        assert_eq!(pkg["name"], "demo");
    }

    #[test]
    fn inspect_package_json_missing_errors() {
        let dir = tempdir().unwrap();
        assert!(inspect_package_json(dir.path()).is_err());
    }
}
