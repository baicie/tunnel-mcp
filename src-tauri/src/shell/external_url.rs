use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::error::ShellError;

/// Phase 6: 桌面壳默认 deny-all，只放行 http/https/mailto。后续若要放开新 scheme，
/// 必须同步更新 `runtime_boundary.rs` 的边界扫描规则。
pub fn validate_external_url(raw_url: &str) -> Result<(), ShellError> {
    let url = url::Url::parse(raw_url)
        .map_err(|_| ShellError::InvalidExternalUrl("url is not valid".to_string()))?;

    match url.scheme() {
        "http" | "https" | "mailto" => Ok(()),
        scheme => Err(ShellError::InvalidExternalUrl(format!(
            "scheme `{scheme}` is not allowed"
        ))),
    }
}

pub fn is_allowed_external_url(raw_url: &str) -> bool {
    validate_external_url(raw_url).is_ok()
}

/// Tauri command 适配层使用的便捷函数：先校验，再通过 opener 插件打开。
pub fn open_external_url(app: &AppHandle, raw_url: &str) -> Result<(), String> {
    validate_external_url(raw_url).map_err(|error| error.to_string())?;

    app.opener()
        .open_url(raw_url.to_string(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_allow_safe_external_urls() {
        assert!(is_allowed_external_url("https://example.com"));
        assert!(is_allowed_external_url("http://localhost:5173"));
        assert!(is_allowed_external_url("mailto:test@example.com"));
    }

    #[test]
    fn should_reject_unsafe_or_unknown_external_urls() {
        assert!(!is_allowed_external_url("local-file-scheme://example"));
        assert!(!is_allowed_external_url("script-scheme://example"));
        assert!(!is_allowed_external_url("unknown-scheme://example"));
        assert!(!is_allowed_external_url("not-a-url"));
    }

    #[test]
    fn validate_should_return_error_for_unknown_scheme() {
        let err = validate_external_url("unknown-scheme://example").unwrap_err();
        assert!(matches!(err, ShellError::InvalidExternalUrl(_)));
    }

    #[test]
    fn validate_should_return_error_for_invalid_url() {
        let err = validate_external_url("not a url").unwrap_err();
        assert!(matches!(err, ShellError::InvalidExternalUrl(_)));
    }
}
