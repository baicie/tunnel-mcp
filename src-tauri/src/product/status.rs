use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub endpoint: Option<String>,
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

pub fn initial_tunnel_status(tunnel_client_path: Option<String>) -> TunnelStatus {
    TunnelStatus {
        installed: tunnel_client_path.is_some_and(|value| !value.is_empty()),
        running: false,
        version: None,
        pid: None,
        endpoint: None,
        last_error: None,
    }
}

pub fn initial_mcp_status() -> McpServerStatus {
    McpServerStatus {
        running: false,
        port: 17891,
        tools: vec![],
        resources: vec![],
    }
}
