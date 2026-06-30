use super::event::LogLevel;
use super::store::AuditLogStore;
use serde_json::Value;

/// Best-effort write of a single audit log event.
///
/// Audit logging must never block or fail a business command. Any
/// `append` error (disk full, file removed, permission denied) is
/// intentionally swallowed here and surfaced only through `log::warn`
/// so the underlying operation still completes.
pub fn append_audit_log(
    store: &AuditLogStore,
    request_id: Option<String>,
    event_type: impl Into<String>,
    level: LogLevel,
    message: impl Into<String>,
    metadata: Value,
) {
    if let Err(err) = store.append(request_id, event_type, level, message, metadata) {
        log::warn!("audit log append failed: {err}");
    }
}
