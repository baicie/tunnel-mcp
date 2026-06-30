use anyhow::{anyhow, Context};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_READ_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDescriptor {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: ResourceKind,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    Filesystem,
}

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
    fn authorized_roots(&self) -> Vec<PathBuf>;
}

#[derive(Debug, Clone)]
pub struct AllowRootsReadPolicy {
    roots: Vec<PathBuf>,
}

impl AllowRootsReadPolicy {
    pub fn new(roots: Vec<PathBuf>) -> anyhow::Result<Self> {
        let mut normalized = Vec::new();

        for root in roots {
            let canonical = root
                .canonicalize()
                .with_context(|| format!("invalid authorized root {}", root.display()))?;

            if !canonical.is_dir() {
                return Err(anyhow!(
                    "authorized root is not a directory: {}",
                    canonical.display()
                ));
            }

            normalized.push(canonical);
        }

        normalized.sort();
        normalized.dedup();

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

    fn authorized_roots(&self) -> Vec<PathBuf> {
        self.roots.clone()
    }
}

pub fn list_authorized_resources(policy: &dyn ReadPolicy) -> Vec<ResourceDescriptor> {
    policy
        .authorized_roots()
        .into_iter()
        .map(|root| {
            let path = root.to_string_lossy().to_string();
            let name = root
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            ResourceDescriptor {
                id: format!("filesystem:{path}"),
                name,
                path,
                kind: ResourceKind::Filesystem,
            }
        })
        .collect()
}

pub fn read_resource(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<ResourceDescriptor> {
    if !policy.can_read(path) {
        return Err(anyhow!("permission denied"));
    }

    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(anyhow!("resource is not a directory"));
    }

    let name = canonical
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| canonical.to_string_lossy().to_string());

    let path = canonical.to_string_lossy().to_string();

    Ok(ResourceDescriptor {
        id: format!("filesystem:{path}"),
        name,
        path,
        kind: ResourceKind::Filesystem,
    })
}

pub fn list_files(path: &Path, policy: &dyn ReadPolicy) -> anyhow::Result<Vec<FileEntry>> {
    if !policy.can_read(path) {
        return Err(anyhow!("permission denied"));
    }

    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(anyhow!("not a directory"));
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(&canonical)? {
        let entry = entry?;
        let path = entry.path();

        if !policy.can_read(&path) {
            continue;
        }

        let meta = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
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

    let canonical = path.canonicalize()?;
    if !canonical.is_file() {
        return Err(anyhow!("not a file"));
    }

    let metadata = fs::metadata(&canonical)?;
    if metadata.len() > MAX_READ_BYTES {
        return Err(anyhow!(
            "file is too large to read safely: {} bytes",
            metadata.len()
        ));
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(&canonical)?.read_to_end(&mut bytes)?;

    let content = String::from_utf8(bytes).map_err(|_| anyhow!("file is not valid utf-8"))?;

    Ok(FileReadResult {
        path: canonical.to_string_lossy().to_string(),
        content,
    })
}
