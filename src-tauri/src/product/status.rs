use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelHealthState {
    Unknown,
    Healthy,
    Warning,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub endpoint: Option<String>,
    pub health: TunnelHealthState,
    pub local_mcp_port_open: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatus {
    pub running: bool,
    pub port: u16,
    pub tools: Vec<String>,
    pub resources: Vec<String>,
}

pub fn initial_tunnel_status(
    tunnel_client_path: Option<String>,
    version: Option<String>,
    local_mcp_port_open: bool,
) -> TunnelStatus {
    TunnelStatus {
        installed: tunnel_client_path.is_some_and(|value| !value.trim().is_empty()),
        running: false,
        version,
        pid: None,
        endpoint: None,
        health: TunnelHealthState::Unknown,
        local_mcp_port_open,
        last_error: None,
    }
}

#[allow(dead_code)]
pub fn initial_mcp_status(port: u16) -> McpServerStatus {
    McpServerStatus {
        running: false,
        port,
        tools: vec![],
        resources: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::{initial_mcp_status, initial_tunnel_status, TunnelHealthState};

    #[test]
    fn tunnel_status_treats_missing_or_blank_path_as_not_installed() {
        assert!(!initial_tunnel_status(None, None, false).installed);
        assert!(!initial_tunnel_status(Some(String::new()), None, false).installed);
        assert!(!initial_tunnel_status(Some("   ".to_string()), None, false).installed);
    }

    #[test]
    fn tunnel_status_keeps_installed_version() {
        let status = initial_tunnel_status(
            Some("/tmp/tunnel-client".to_string()),
            Some("0.2.0".to_string()),
            false,
        );

        assert!(status.installed);
        assert_eq!(status.version, Some("0.2.0".to_string()));
        assert_eq!(status.health, TunnelHealthState::Unknown);
    }

    #[test]
    fn mcp_status_is_stopped_with_configured_port() {
        let status = initial_mcp_status(18888);

        assert!(!status.running);
        assert_eq!(status.port, 18888);
        assert!(status.tools.is_empty());
        assert!(status.resources.is_empty());
    }
}
