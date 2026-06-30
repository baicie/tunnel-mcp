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

    #[test]
    fn resources_list_returns_authorized_roots_not_global_filesystem() {
        let dir = tempdir().unwrap();
        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "resources/list".to_string(),
                params: json!({}),
            },
            &policy,
        );

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let resources = result["resources"].as_array().unwrap();

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0]["kind"], json!("filesystem"));
    }

    #[test]
    fn resources_read_does_not_read_file_contents() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "hello").unwrap();

        let policy = AllowRootsReadPolicy::new(vec![dir.path().to_path_buf()]).unwrap();

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "resources/read".to_string(),
                params: json!({ "path": file }),
            },
            &policy,
        );

        assert!(response.error.is_some());
    }
}
