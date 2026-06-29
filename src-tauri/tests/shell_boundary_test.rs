use desktop_shell::shell::runtime_boundary::SHELL_FORBIDDEN_MARKERS;
use std::fs;
use std::path::{Path, PathBuf};

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

#[test]
fn src_tauri_src_should_not_contain_legacy_business_markers() {
    let root = manifest_dir().join("src");
    let mut files = Vec::new();

    walk(&root, &mut files);

    let allow_list = ["src/shell/runtime_boundary.rs"];

    let mut violations = Vec::new();

    for file in files {
        let rel = file
            .strip_prefix(manifest_dir())
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");

        if allow_list.contains(&rel.as_str()) {
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
