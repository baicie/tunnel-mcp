use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub const DEFAULT_LOCAL_MCP_HOST: &str = "127.0.0.1";
pub const DEFAULT_LOCAL_MCP_PATH: &str = "/mcp";

pub fn local_mcp_url(port: u16) -> String {
    format!("http://{DEFAULT_LOCAL_MCP_HOST}:{port}{DEFAULT_LOCAL_MCP_PATH}")
}

pub fn is_local_port_open(port: u16) -> bool {
    let addr: SocketAddr = match format!("{DEFAULT_LOCAL_MCP_HOST}:{port}").parse() {
        Ok(value) => value,
        Err(_) => return false,
    };

    TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok()
}

#[allow(dead_code)]
pub fn local_mcp_port_hint(port: u16) -> Option<String> {
    if is_local_port_open(port) {
        Some(format!(
            "local MCP port {port} is already reachable. In Phase 2 this may mean another process is using the port; in later phases it may be the embedded MCP server."
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::local_mcp_url;

    #[test]
    fn builds_local_mcp_url_from_configured_port() {
        assert_eq!(local_mcp_url(18888), "http://127.0.0.1:18888/mcp");
    }
}
