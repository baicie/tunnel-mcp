use std::fs;
use std::path::PathBuf;
use tunnel_mcp::shell::brand::APP_BRAND;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn package_json_should_match_rust_brand() {
    let package_json_path = repo_root().join("package.json");
    let package_json = fs::read_to_string(package_json_path).expect("read package.json");

    assert!(
        package_json.contains(&format!("\"name\": \"{}\"", APP_BRAND.package_name)),
        "package.json name should match APP_BRAND.package_name"
    );

    assert!(
        package_json.contains(APP_BRAND.description),
        "package.json description should match APP_BRAND.description"
    );
}

#[test]
fn tauri_config_should_match_rust_brand() {
    let tauri_config_path = repo_root().join("src-tauri/tauri.conf.json");
    let tauri_config = fs::read_to_string(tauri_config_path).expect("read tauri.conf.json");

    assert!(
        tauri_config.contains(&format!("\"productName\": \"{}\"", APP_BRAND.product_name)),
        "tauri productName should match APP_BRAND.product_name"
    );

    assert!(
        tauri_config.contains(&format!("\"identifier\": \"{}\"", APP_BRAND.identifier)),
        "tauri identifier should match APP_BRAND.identifier"
    );

    assert!(
        tauri_config.contains(APP_BRAND.updater_endpoint),
        "tauri updater endpoint should match APP_BRAND.updater_endpoint"
    );
}

#[test]
fn cargo_toml_should_match_rust_brand() {
    let cargo_toml_path = repo_root().join("src-tauri/Cargo.toml");
    let cargo_toml = fs::read_to_string(cargo_toml_path).expect("read Cargo.toml");

    assert!(
        cargo_toml.contains(&format!("name = \"{}\"", APP_BRAND.package_name)),
        "Cargo.toml package name should match APP_BRAND.package_name"
    );

    assert!(
        cargo_toml.contains(APP_BRAND.description),
        "Cargo.toml description should match APP_BRAND.description"
    );
}
