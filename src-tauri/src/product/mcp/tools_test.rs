#[cfg(test)]
mod tests {
    use crate::product::mcp::protocol::JsonRpcRequest;
    use crate::product::mcp::resources::AllowRootsReadPolicy;
    use crate::product::mcp::tools::handle_request;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn tools_list_returns_mvp_tools() {
        let dir = tempdir().unwrap();
        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();
        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "tools/list".to_string(),
                params: json!({}),
            },
            &policy,
        );
        assert!(response.error.is_none());
        assert!(response.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .contains(&json!("files/read")));
    }

    #[test]
    fn unknown_method_returns_error() {
        let dir = tempdir().unwrap();
        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();
        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "command/run".to_string(),
                params: json!({}),
            },
            &policy,
        );
        assert!(response.error.is_some());
    }
}
