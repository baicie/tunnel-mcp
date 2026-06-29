use tunnel_mcp::shell::external_url::{is_allowed_external_url, validate_external_url};

#[test]
fn should_allow_safe_external_urls() {
    assert!(is_allowed_external_url("https://example.com"));
    assert!(is_allowed_external_url("http://localhost:5173"));
    assert!(is_allowed_external_url("mailto:test@example.com"));
}

#[test]
fn should_reject_unsafe_external_urls() {
    assert!(!is_allowed_external_url("unknown-scheme://example"));
    assert!(!is_allowed_external_url("local-file-scheme://tmp/a.txt"));
    assert!(!is_allowed_external_url("script-scheme://alert"));
    assert!(!is_allowed_external_url("not a url"));
}

#[test]
fn validation_should_return_error_for_invalid_url() {
    assert!(validate_external_url("unknown-scheme://example").is_err());
}

#[test]
fn validation_should_return_error_for_malformed_url() {
    assert!(validate_external_url("").is_err());
    assert!(validate_external_url("not a url").is_err());
}
