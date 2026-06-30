#[cfg(test)]
mod tests {
    use crate::product::mcp::protocol::JsonRpcRequest;
    use crate::product::mcp::tools::handle_request;
    use crate::product::permissions::read_policy::PermissionReadPolicy;
    use crate::product::permissions::scope::{PermissionAccess, PermissionKind, PermissionScope};
    use serde_json::json;
    use tempfile::tempdir;

    fn policy_for_dir(dir: &std::path::Path) -> PermissionReadPolicy {
        PermissionReadPolicy::new(vec![PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: dir.display().to_string(),
            access: PermissionAccess::Read,
            require_approval: false,
        }])
        .unwrap()
    }

    #[test]
    fn tools_list_returns_mvp_tools() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());
        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "tools/list".to_string(),
                params: json!({}),
            },
            &policy,
            None,
        );
        assert!(response.error.is_none());
        assert!(response.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .contains(&json!("files/read")));
    }

    #[test]
    fn tools_list_includes_files_write() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "tools/list".to_string(),
                params: json!({}),
            },
            &policy,
            None,
        );

        let tools = response.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .clone();

        assert!(tools.contains(&json!("files.write")));
    }

    #[test]
    fn unknown_method_returns_error() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());
        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "command/run".to_string(),
                params: json!({}),
            },
            &policy,
            None,
        );
        assert!(response.error.is_some());
    }

    #[test]
    fn resources_list_returns_authorized_roots_not_global_filesystem() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "resources/list".to_string(),
                params: json!({}),
            },
            &policy,
            None,
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

        let policy = policy_for_dir(dir.path());

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "resources/read".to_string(),
                params: json!({ "path": file }),
            },
            &policy,
            None,
        );

        assert!(response.error.is_some());
    }

    #[test]
    fn files_write_without_context_returns_error() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "files.write".to_string(),
                params: json!({ "path": dir.path().join("a.txt"), "content": "x" }),
            },
            &policy,
            None,
        );

        assert!(response.error.is_some());
    }
}
