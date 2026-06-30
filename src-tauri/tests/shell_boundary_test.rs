use std::fs;
use std::path::{Path, PathBuf};
use tunnel_mcp::shell::runtime_boundary::SHELL_FORBIDDEN_MARKERS;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if ["target", ".git"].contains(&file_name) {
            continue;
        }

        if path.is_dir() {
            walk(&path, files);
        } else {
            files.push(path);
        }
    }
}

/// Paths that legitimately reference the shell-only `runtime_boundary`
/// vocabulary. The boundary scanner and the runtime boundary module
/// itself must mention every marker; we exclude them from the scan.
fn is_marker_aware(rel: &str) -> bool {
    rel == "src/shell/runtime_boundary.rs" || rel.starts_with("src/shell/runtime_boundary")
}

/// The Tunnel MCP product layer is allowed to name its own features.
/// Markers like `tunnel`, `mcp`, `provider` are intrinsic to the
/// product namespace and only checked outside the product directory.
fn is_product_path(rel: &str) -> bool {
    rel.starts_with("src/product/")
        || rel.starts_with("src/product")
        || rel.starts_with("src/commands/tunnel")
        || rel.starts_with("src/commands/workspace")
}

/// Shell boundary files that legitimately reference the product namespace.
fn is_shell_boundary_whitelisted(rel: &str) -> bool {
    rel == "src/lib.rs" || rel == "src/commands/mod.rs"
}

#[test]
fn src_tauri_src_should_not_contain_legacy_business_markers() {
    let root = manifest_dir().join("src");
    let mut files = Vec::new();

    walk(&root, &mut files);

    let mut violations = Vec::new();

    for file in files {
        let rel = file
            .strip_prefix(manifest_dir())
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");

        if is_marker_aware(&rel) {
            continue;
        }

        if is_product_path(&rel) || is_shell_boundary_whitelisted(&rel) {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        let lower = content.to_lowercase();

        for marker in SHELL_FORBIDDEN_MARKERS {
            if lower.contains(&marker.to_lowercase()) {
                violations.push(format!("{rel}: {marker}"));
            }
        }
    }

    assert_eq!(violations, Vec::<String>::new());
}
