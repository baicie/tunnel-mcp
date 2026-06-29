use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use super::brand::APP_BRAND;

/// Phase 6: 桌面壳配置目录统一以 `APP_BRAND.config_dir_name` 收口，避免不同平台
/// 默认目录名混淆。返回路径可能自动追加品牌子目录。
pub fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;

    Ok(normalize_brand_dir(base))
}

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    Ok(normalize_brand_dir(base))
}

pub fn settings_path_from_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(APP_BRAND.settings_file_name)
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(settings_path_from_dir(&app_config_dir(app)?))
}

pub fn log_file_name() -> &'static str {
    APP_BRAND.log_file_name
}

fn normalize_brand_dir(path: PathBuf) -> PathBuf {
    if path.ends_with(APP_BRAND.config_dir_name) {
        return path;
    }

    path.join(APP_BRAND.config_dir_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_brand_dir_should_append_brand_when_missing() {
        let path = normalize_brand_dir(PathBuf::from("/tmp/base"));
        assert!(path.ends_with(APP_BRAND.config_dir_name));
    }

    #[test]
    fn normalize_brand_dir_should_not_append_twice() {
        let input = PathBuf::from("/tmp/base").join(APP_BRAND.config_dir_name);
        let path = normalize_brand_dir(input.clone());
        assert_eq!(path, input);
    }

    #[test]
    fn should_build_settings_path() {
        let path =
            settings_path_from_dir(Path::new(&format!("/tmp/{}", APP_BRAND.config_dir_name)));
        assert!(path.ends_with(APP_BRAND.settings_file_name));
    }

    #[test]
    fn log_file_name_should_use_brand() {
        assert_eq!(log_file_name(), APP_BRAND.log_file_name);
    }
}
