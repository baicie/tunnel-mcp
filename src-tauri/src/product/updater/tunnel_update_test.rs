#[cfg(test)]
mod tests {
    use super::super::tunnel_update::{
        current_version, previous_version_dir, rollback_tunnel_client, version_from_managed_path,
    };
    use crate::product::settings::TunnelSettings;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extracts_version_from_managed_binary_path() {
        assert_eq!(
            version_from_managed_path("/app/bin/0.1.0/tunnel-client"),
            Some("0.1.0".to_string())
        );
    }

    #[test]
    fn unconventional_path_does_not_fake_version() {
        assert_eq!(
            version_from_managed_path("/usr/local/bin/tunnel-client"),
            None
        );
    }

    #[test]
    fn current_version_prefers_settings_version() {
        let settings = TunnelSettings {
            tunnel_client_version: Some("0.3.0".to_string()),
            tunnel_client_path: Some("/usr/local/bin/tunnel-client".to_string()),
            ..TunnelSettings::default()
        };

        assert_eq!(current_version(&settings), Some("0.3.0".to_string()));
    }

    #[test]
    fn previous_version_dir_returns_none_when_bin_missing() {
        let dir = tempdir().unwrap();
        let previous = previous_version_dir(dir.path(), "0.2.0").unwrap();
        assert!(previous.is_none());
    }

    #[test]
    fn finds_highest_previous_version_semver_like() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin/0.2.0")).unwrap();
        fs::create_dir_all(dir.path().join("bin/0.10.0")).unwrap();
        fs::write(dir.path().join("bin/0.2.0/tunnel-client"), "old").unwrap();
        fs::write(dir.path().join("bin/0.10.0/tunnel-client"), "newer").unwrap();

        let previous = previous_version_dir(dir.path(), "0.11.0").unwrap().unwrap();

        assert!(previous.ends_with("0.10.0"));
    }

    #[test]
    fn rollback_ignores_previous_dir_without_binary() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin/0.1.0")).unwrap();
        fs::create_dir_all(dir.path().join("bin/0.2.0")).unwrap();

        let mut settings = TunnelSettings {
            tunnel_client_path: Some(
                dir.path()
                    .join("bin/0.2.0/tunnel-client")
                    .to_string_lossy()
                    .to_string(),
            ),
            tunnel_client_version: Some("0.2.0".to_string()),
            ..TunnelSettings::default()
        };

        let status = rollback_tunnel_client(dir.path(), &mut settings).unwrap();

        assert_eq!(status.current_version, Some("0.2.0".to_string()));
        assert!(settings
            .tunnel_client_path
            .as_deref()
            .unwrap()
            .contains("0.2.0"));
    }

    #[test]
    fn rollback_updates_settings_path_and_version() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin/0.1.0")).unwrap();
        fs::create_dir_all(dir.path().join("bin/0.2.0")).unwrap();
        fs::write(dir.path().join("bin/0.1.0/tunnel-client"), "old").unwrap();

        let mut settings = TunnelSettings {
            tunnel_client_path: Some(
                dir.path()
                    .join("bin/0.2.0/tunnel-client")
                    .to_string_lossy()
                    .to_string(),
            ),
            tunnel_client_version: Some("0.2.0".to_string()),
            ..TunnelSettings::default()
        };

        let status = rollback_tunnel_client(dir.path(), &mut settings).unwrap();

        assert_eq!(status.current_version, Some("0.1.0".to_string()));
        assert!(status.installed);
        assert!(!status.update_available);
        assert!(settings
            .tunnel_client_path
            .as_deref()
            .unwrap()
            .contains("0.1.0"));
        assert_eq!(settings.tunnel_client_version, Some("0.1.0".to_string()));
    }

    #[test]
    fn rollback_without_installed_version_reports_not_installed() {
        let dir = tempdir().unwrap();
        let mut settings = TunnelSettings::default();

        let status = rollback_tunnel_client(dir.path(), &mut settings).unwrap();

        assert!(!status.installed);
        assert!(status.current_version.is_none());
    }

    #[test]
    fn rollback_without_previous_version_keeps_current() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin/0.2.0")).unwrap();
        fs::write(dir.path().join("bin/0.2.0/tunnel-client"), "current").unwrap();

        let mut settings = TunnelSettings {
            tunnel_client_path: Some(
                dir.path()
                    .join("bin/0.2.0/tunnel-client")
                    .to_string_lossy()
                    .to_string(),
            ),
            tunnel_client_version: Some("0.2.0".to_string()),
            ..TunnelSettings::default()
        };

        let status = rollback_tunnel_client(dir.path(), &mut settings).unwrap();

        assert!(status.installed);
        assert_eq!(status.current_version, Some("0.2.0".to_string()));
        assert!(!status.update_available);
    }
}
