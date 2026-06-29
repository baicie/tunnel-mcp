use anyhow::{anyhow, Context};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub kind: FileEntryKind,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileEntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileReadResult {
    pub path: String,
    pub content: String,
}

pub trait ReadPolicy: Send + Sync + 'static {
    fn can_read(&self, path: &Path) -> bool;
}

#[derive(Debug, Clone)]
pub struct AllowRootsReadPolicy {
    roots: Vec<PathBuf>,
}

impl AllowRootsReadPolicy {
    pub fn new(roots: Vec<PathBuf>) -> anyhow::Result<Self> {
        let mut normalized = Vec::new();
        for root in roots {
            normalized.push(
                root.canonicalize()
                    .with_context(|| format!("invalid root {}", root.display()))?,
            );
        }
        Ok(Self { roots: normalized })
    }
}

impl ReadPolicy for AllowRootsReadPolicy {
    fn can_read(&self, path: &Path) -> bool {
        let Ok(canonical) = path.canonicalize() else {
            return false;
        };
        self.roots.iter().any(|root| canonical.starts_with(root))
    }
}

pub fn list_files(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<Vec<FileEntry>> {
    if !policy.can_read(path) {
        return Err(anyhow!("permission denied"));
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            kind: if meta.is_dir() {
                FileEntryKind::Directory
            } else {
                FileEntryKind::File
            },
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub fn read_file(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<FileReadResult> {
    if !policy.can_read(path) {
        return Err(anyhow!("permission denied"));
    }
    if !path.is_file() {
        return Err(anyhow!("not a file"));
    }
    let content = fs::read_to_string(path)?;
    Ok(FileReadResult {
        path: path.to_string_lossy().to_string(),
        content,
    })
}
