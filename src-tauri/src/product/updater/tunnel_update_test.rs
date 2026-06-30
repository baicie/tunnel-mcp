#[cfg(test)]
mod tests {
    use super::super::tunnel_update::{
        previous_version_dir, rollback_tunnel_client, version_from_path,
    };
    use crate::product::settings::TunnelSettings;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extracts_version_from_binary_path() {
        assert_eq!(
            version_from_path("/app/bin/0.1.0/tunnel-client"),
            Some("0.1.0".to_string())
        );
    }

    #[test]
    fn returns_version_segment_for_unconventional_path() {
        // `version_from_path` simply reads the parent directory's
        // file_name. For paths that do not match the
        // `<bin_root>/<version>/tunnel-client` convention it falls
        // back to whatever the directory name is.
        assert_eq!(
            version_from_path("/usr/local/bin/tunnel-client"),
            Some("bin".to_string())
        );
    }

    #[test]
    fn previous_version_dir_returns_none_when_bin_missing() {
        let dir = tempdir().unwrap();
        let previous = previous_version_dir(dir.path(), "0.2.0").unwrap();
        assert!(previous.is_none());
    }

    #[test]
    fn finds_previous_version_dir() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin/0.1.0")).unwrap();
        fs::create_dir_all(dir.path().join("bin/0.2.0")).unwrap();
        let previous = previous_version_dir(dir.path(), "0.2.0").unwrap().unwrap();
        assert!(previous.ends_with("0.1.0"));
    }

    #[test]
    fn rollback_updates_settings_path() {
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

        let mut settings = TunnelSettings {
            tunnel_client_path: Some(
                dir.path()
                    .join("bin/0.2.0/tunnel-client")
                    .to_string_lossy()
                    .to_string(),
            ),
            ..TunnelSettings::default()
        };

        let status = rollback_tunnel_client(dir.path(), &mut settings).unwrap();

        assert!(status.installed);
        assert_eq!(status.current_version, Some("0.2.0".to_string()));
        assert!(!status.update_available);
    }
}
