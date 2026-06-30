use serde::Serialize;

/// Result of checking for a newer desktop-shell release.
///
/// Phase 8 ships a placeholder until the Tauri updater is wired with
/// a signed `latest.json` endpoint. The placeholder still surfaces the
/// currently running version so the UI can render an explicit "no
/// update available yet" message instead of pretending the check ran.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub notes: Option<String>,
}

/// Build the placeholder "no update" result. We use this until the
/// Tauri updater is wired with signing keys and a `latest.json`
/// release pipeline.
pub fn no_update(current_version: impl Into<String>) -> UpdateCheckResult {
    UpdateCheckResult {
        available: false,
        current_version: current_version.into(),
        latest_version: None,
        notes: Some(
            "Tauri updater integration placeholder; enable after signing and release pipeline are configured."
                .to_string(),
        ),
    }
}
