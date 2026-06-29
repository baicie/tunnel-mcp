use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub available: bool,
    pub version: Option<String>,
    pub notes: Option<String>,
}

/// Phase 6: 桌面壳仅启用 `tauri-plugin-updater`，不再读取业务 endpoint / 校验签。
pub fn plugin<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.plugin(tauri_plugin_updater::Builder::new().build())
}

pub fn check_update_noop() -> UpdateCheckResult {
    UpdateCheckResult {
        available: false,
        version: None,
        notes: None,
    }
}

pub fn updater_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_check_noop_should_return_no_update() {
        let result = check_update_noop();

        assert!(!result.available);
        assert_eq!(result.version, None);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn updater_should_be_enabled_by_default() {
        assert!(updater_enabled());
    }
}
