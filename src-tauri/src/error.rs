use std::fmt;

/// Phase 6: 桌面壳最小错误类型。
///
/// 旧业务错误枚举（带业务校验 / 业务本地化文案 / 业务专用变体）已被物理删除，
/// 壳代码不再引入任何业务错误分类。
#[derive(Debug)]
pub enum ShellError {
    Io(String),
    Json(String),
    InvalidExternalUrl(String),
    RuntimeBoundary(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "io error: {message}"),
            Self::Json(message) => write!(formatter, "json error: {message}"),
            Self::InvalidExternalUrl(message) => {
                write!(formatter, "invalid external url: {message}")
            }
            Self::RuntimeBoundary(message) => {
                write!(formatter, "runtime boundary error: {message}")
            }
        }
    }
}

impl std::error::Error for ShellError {}

impl From<std::io::Error> for ShellError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for ShellError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

impl From<ShellError> for String {
    fn from(error: ShellError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_should_include_variant_message() {
        let err = ShellError::InvalidExternalUrl("bad scheme".to_string());
        assert_eq!(err.to_string(), "invalid external url: bad scheme");
    }

    #[test]
    fn runtime_boundary_display_should_include_message() {
        let err = ShellError::RuntimeBoundary("blocked command".to_string());
        assert_eq!(err.to_string(), "runtime boundary error: blocked command");
    }

    #[test]
    fn io_error_should_convert_to_shell_error() {
        let io = std::io::Error::other("disk full");
        let shell: ShellError = io.into();
        assert!(matches!(shell, ShellError::Io(_)));
    }

    #[test]
    fn json_error_should_convert_to_shell_error() {
        let json: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let shell: ShellError = json.into();
        assert!(matches!(shell, ShellError::Json(_)));
    }
}
