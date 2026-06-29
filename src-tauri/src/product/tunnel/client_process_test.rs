#[cfg(test)]
mod tests {
    use crate::product::settings::{LogLevel, TunnelSettings};
    use crate::product::tunnel::client_process::{TunnelClientLogLine, TunnelProcessManager};

    fn settings_with_path() -> TunnelSettings {
        TunnelSettings {
            openai_api_key: None,
            tunnel_id: None,
            tunnel_client_path: Some("/tmp/tunnel-client".to_string()),
            tunnel_client_version: Some("0.2.0".to_string()),
            resource_root: None,
            mcp_server_port: 17891,
            log_level: LogLevel::Info,
            auto_start: false,
            auto_update_tunnel_client: true,
        }
    }

    #[test]
    fn status_reports_not_installed_without_path() {
        let manager = TunnelProcessManager::default();
        let status = manager.status(&TunnelSettings::default()).unwrap();

        assert!(!status.installed);
        assert!(!status.running);
        assert!(status.pid.is_none());
    }

    #[test]
    fn status_reports_installed_version_from_settings() {
        let manager = TunnelProcessManager::default();
        let status = manager.status(&settings_with_path()).unwrap();

        assert!(status.installed);
        assert_eq!(status.version, Some("0.2.0".to_string()));
    }

    #[test]
    fn start_requires_binary_path() {
        let manager = TunnelProcessManager::default();
        let result = manager.start(&TunnelSettings::default());

        assert!(result.is_err());
    }

    #[test]
    fn start_requires_tunnel_id_and_openai_key() {
        let manager = TunnelProcessManager::default();
        let mut settings = settings_with_path();

        let err = manager.start(&settings).unwrap_err().to_string();
        assert!(
            err.contains("tunnel id is not configured"),
            "unexpected error: {err}"
        );

        settings.tunnel_id = Some("tun_test".to_string());
        let err = manager.start(&settings).unwrap_err().to_string();
        assert!(
            err.contains("OpenAI key is not configured"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn logs_are_initially_empty() {
        let manager = TunnelProcessManager::default();
        let logs: Vec<TunnelClientLogLine> = manager.logs();

        assert!(logs.is_empty());
    }
}
