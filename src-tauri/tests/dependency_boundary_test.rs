use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn strip_inline_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut previous = '\0';

    for (index, current) in line.char_indices() {
        if current == '\'' && !in_double {
            in_single = !in_single;
        }

        if current == '"' && !in_single && previous != '\\' {
            in_double = !in_double;
        }

        if current == '#' && !in_single && !in_double {
            return line[..index].trim().to_string();
        }

        previous = current;
    }

    line.trim().to_string()
}

fn normalize_dependency_name(name: &str) -> String {
    name.trim().trim_matches('"').trim_matches('\'').to_string()
}

fn parse_package_value(line: &str) -> Option<String> {
    let package_index = line.find("package")?;
    let after_package = &line[package_index + "package".len()..];
    let equal_index = after_package.find('=')?;
    let value = after_package[equal_index + 1..].trim();
    let quote = value.chars().next()?;

    if quote != '"' && quote != '\'' {
        return None;
    }

    let rest = &value[quote.len_utf8()..];
    let end = rest.find(quote)?;

    Some(rest[..end].to_string())
}

fn dependency_table_name(section: &str) -> Option<String> {
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(rest) = section.strip_prefix(&format!("{key}.")) {
            return Some(normalize_dependency_name(rest));
        }

        let marker = format!(".{key}.");
        if let Some(index) = section.find(&marker) {
            return Some(normalize_dependency_name(&section[index + marker.len()..]));
        }
    }

    None
}

fn is_dependency_list_section(section: &str) -> bool {
    matches!(
        section,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    ) || section.ends_with(".dependencies")
        || section.ends_with(".dev-dependencies")
        || section.ends_with(".build-dependencies")
}

fn parse_cargo_dependency_names(content: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut current_section = String::new();

    for raw_line in content.lines() {
        let line = strip_inline_comment(raw_line);

        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();

                if let Some(name) = dependency_table_name(&current_section) {
                    names.insert(name);
                }
            }

            continue;
        }

        if let Some(package_name) = parse_package_value(&line) {
            names.insert(package_name);
        }

        if !is_dependency_list_section(&current_section) {
            continue;
        }

        let Some((name, _value)) = line.split_once('=') else {
            continue;
        };

        let name = normalize_dependency_name(name);

        if !name.is_empty() && name != "package" {
            names.insert(name);
        }
    }

    names
}

#[test]
#[ignore = "template-only check; requires `--ignored` in product forks"]
fn cargo_toml_should_not_contain_legacy_business_dependencies() {
    let cargo_toml =
        fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("read Cargo.toml");

    let dependency_names = parse_cargo_dependency_names(&cargo_toml);

    let forbidden = [
        "anyhow",
        "arboard",
        "auto-launch",
        "base64",
        "chrono",
        "dirs",
        "log",
        "once_cell",
        "serde_yaml",
        "serial_test",
        "tauri-plugin-deep-link",
        "tauri-plugin-single-instance",
        "thiserror",
        "tokio",
        "toml",
        "toml_edit",
        "webkit2gtk",
        "winreg",
        "windows-sys",
        "objc2",
        "objc2-app-kit",
        "reqwest",
        "axum",
        "tower",
        "tower-http",
        "hyper",
        "hyper-util",
        "hyper-rustls",
        "http",
        "http-body",
        "http-body-util",
        "httparse",
        "tokio-rustls",
        "rustls",
        "webpki-roots",
        "rustls-native-certs",
        "regex",
        "rquickjs",
        "zip",
        "flate2",
        "brotli",
        "zstd",
        "rusqlite",
        "indexmap",
        "rust_decimal",
        "uuid",
        "sha2",
        "hmac",
        "json5",
        "json-five",
        "bytes",
        "async-stream",
        "futures",
    ];

    let violations = forbidden
        .iter()
        .filter(|dep| dependency_names.contains(**dep))
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(
        violations,
        Vec::<&str>::new(),
        "Cargo.toml contains legacy business dependencies"
    );
}

#[test]
fn cargo_toml_should_keep_shell_dependencies() {
    let cargo_toml =
        fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("read Cargo.toml");

    let dependency_names = parse_cargo_dependency_names(&cargo_toml);

    for required in [
        "serde",
        "serde_json",
        "tauri",
        "tauri-plugin-log",
        "tauri-plugin-opener",
        "tauri-plugin-process",
        "tauri-plugin-dialog",
        "tauri-plugin-store",
        "tauri-plugin-updater",
        "tauri-plugin-window-state",
        "url",
    ] {
        assert!(
            dependency_names.contains(required),
            "missing required shell dependency: {required}"
        );
    }
}
