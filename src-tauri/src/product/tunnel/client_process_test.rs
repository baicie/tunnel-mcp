#[cfg(test)]
mod tests {
    use super::super::client_process::TunnelProcessManager;
    use crate::product::settings::TunnelSettings;

    #[test]
    fn status_reports_not_installed_without_path() {
        let manager = TunnelProcessManager::default();
        let status = manager.status(&TunnelSettings::default()).unwrap();
        assert!(!status.installed);
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.last_error.is_none());
    }

    #[test]
    fn start_requires_binary_path() {
        let manager = TunnelProcessManager::default();
        let result = manager.start(&TunnelSettings::default(), "http://127.0.0.1:17891/mcp");
        assert!(result.is_err());
    }

    #[test]
    fn start_requires_tunnel_id_and_openai_key() {
        let manager = TunnelProcessManager::default();
        let mut settings = TunnelSettings::default();
        settings.tunnel_client_path = Some("/tmp/tunnel-client".to_string());

        let err = manager
            .start(&settings, "http://127.0.0.1:17891/mcp")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("tunnel id is not configured"),
            "unexpected error: {err}"
        );

        settings.tunnel_id = Some("tun_test".to_string());
        let err = manager
            .start(&settings, "http://127.0.0.1:17891/mcp")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("OpenAI key is not configured"),
            "unexpected error: {err}"
        );
    }
}
