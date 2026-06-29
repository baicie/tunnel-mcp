use desktop_shell::shell::settings_store::{
    load_settings_from_path, save_settings_to_path, ShellSettings, ThemeMode,
};
use tempfile::tempdir;

#[test]
fn missing_settings_should_return_default() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("shell-settings.json");

    let settings = load_settings_from_path(&path).expect("load settings");

    assert_eq!(settings, ShellSettings::default());
}

#[test]
fn settings_should_round_trip() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("shell-settings.json");

    let settings = ShellSettings {
        theme: ThemeMode::Dark,
        start_minimized: true,
    };

    save_settings_to_path(&path, &settings).expect("save settings");

    let loaded = load_settings_from_path(&path).expect("load settings");

    assert_eq!(loaded, settings);
}

#[test]
fn invalid_settings_json_should_fail() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("shell-settings.json");

    std::fs::write(&path, "{ invalid json").expect("write invalid json");

    assert!(load_settings_from_path(&path).is_err());
}

#[test]
fn save_should_create_parent_directory() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("nested").join("shell-settings.json");

    save_settings_to_path(&path, &ShellSettings::default()).expect("save settings");

    assert!(path.exists());
}
