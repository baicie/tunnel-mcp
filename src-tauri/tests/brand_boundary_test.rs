use desktop_shell::shell::brand::{app_title, brand_values, APP_BRAND};

#[test]
fn brand_should_have_template_identity() {
    assert!(!APP_BRAND.app_name.is_empty());
    assert!(!APP_BRAND.package_name.is_empty());
    assert!(!APP_BRAND.product_name.is_empty());
    assert!(!APP_BRAND.identifier.is_empty());
    assert!(!APP_BRAND.repository_url.is_empty());
    assert!(!APP_BRAND.deep_link_scheme.is_empty());
    assert!(!APP_BRAND.updater_endpoint.is_empty());
}

#[test]
fn brand_should_drive_derived_file_names() {
    assert_eq!(
        APP_BRAND.database_file_name,
        format!("{}.db", APP_BRAND.package_name)
    );

    assert_eq!(
        APP_BRAND.log_file_name,
        format!("{}.log", APP_BRAND.package_name)
    );

    assert_eq!(APP_BRAND.config_dir_name, APP_BRAND.package_name);
}

#[test]
fn app_title_should_use_product_name() {
    assert_eq!(app_title(None), APP_BRAND.window_title);
    assert_eq!(
        app_title(Some("Settings")),
        format!("Settings - {}", APP_BRAND.window_title)
    );
}

#[test]
fn brand_values_should_not_be_empty() {
    for value in brand_values() {
        assert!(!value.trim().is_empty());
    }
}
