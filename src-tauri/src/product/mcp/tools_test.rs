#[cfg(test)]
mod tests {
    use crate::product::approvals::store::ApprovalStore;
    use crate::product::approvals::write_log::WriteLogStore;
    use crate::product::logs::event::{ListLogsInput, LogLevel};
    use crate::product::logs::store::AuditLogStore;
    use crate::product::mcp::protocol::JsonRpcRequest;
    use crate::product::mcp::tools::{handle_request, McpWriteContext};
    use crate::product::permissions::policy::PermissionPolicy;
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

    fn write_context_for(dir: &std::path::Path) -> McpWriteContext {
        McpWriteContext {
            permission_policy: PermissionPolicy::new(vec![]).unwrap(),
            approval_store: ApprovalStore::new(dir.join("approvals.json")),
            write_log_store: WriteLogStore::new(dir.join("write-log.json")),
            audit_log_store: AuditLogStore::new(dir.join("logs.ndjson")),
        }
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

    #[test]
    fn mcp_request_writes_audit_log_with_request_id() {
        let dir = tempdir().unwrap();
        let policy = policy_for_dir(dir.path());
        let context = write_context_for(dir.path());

        let _ = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("req-1"),
                method: "tools/list".to_string(),
                params: json!({}),
            },
            &policy,
            Some(&context),
        );

        let logs = context
            .audit_log_store
            .list(ListLogsInput {
                r#type: Some("mcp.request".to_string()),
                request_id: Some("req-1".to_string()),
                limit: Some(10),
            })
            .unwrap();

        assert!(logs
            .iter()
            .any(|event| event.r#type == "mcp.request"
                && event.request_id.as_deref() == Some("req-1")));
    }

    #[test]
    fn files_read_writes_file_read_audit_event() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "hello").unwrap();

        let policy = policy_for_dir(dir.path());
        let context = write_context_for(dir.path());

        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("req-read"),
                method: "files/read".to_string(),
                params: json!({ "path": file.to_string_lossy() }),
            },
            &policy,
            Some(&context),
        );

        assert!(response.error.is_none());

        let logs = context
            .audit_log_store
            .list(ListLogsInput {
                r#type: Some("file.read".to_string()),
                request_id: Some("req-read".to_string()),
                limit: Some(10),
            })
            .unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
    }
}
