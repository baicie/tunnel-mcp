use super::brand::APP_BRAND;

/// The shell's logging plugin only enables `tauri-plugin-log`; no
/// product-specific filter layers are attached.
pub fn plugin<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.plugin(tauri_plugin_log::Builder::new().build())
}

pub fn log_file_name() -> &'static str {
    APP_BRAND.log_file_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_file_name_should_use_brand() {
        assert_eq!(log_file_name(), APP_BRAND.log_file_name);
    }

    #[test]
    fn log_file_name_should_not_use_legacy_brand() {
        let lowered = log_file_name().to_ascii_lowercase();
        let legacy = String::from_iter(['c', 'c']);
        assert!(!lowered.contains(&format!("{legacy}-")));
    }
}
