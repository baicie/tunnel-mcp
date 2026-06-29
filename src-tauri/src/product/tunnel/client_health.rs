/// Default local MCP endpoint that the tunnel-client proxies.
///
/// Phase 2 only needs a stable URL so the spawned binary knows where
/// to forward the upstream MCP traffic. The actual server lifecycle
/// lives in a later phase.
pub const DEFAULT_LOCAL_MCP_URL: &str = "http://127.0.0.1:17891/mcp";
