use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::{
    list_authorized_resources, list_files, read_file, read_resource, ReadPolicy,
};
use super::write_tools::{handle_files_write, WriteToolResult};
use crate::product::approvals::store::ApprovalStore;
use crate::product::approvals::write_log::WriteLogStore;
use crate::product::permissions::policy::PermissionPolicy;
use log::warn;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const MCP_TOOLS: &[&str] = &[
    "resources/list",
    "resources/read",
    "files/list",
    "files/read",
    "files.write",
];

pub struct McpWriteContext {
    pub permission_policy: PermissionPolicy,
    pub approval_store: ApprovalStore,
    pub write_log_store: WriteLogStore,
}

pub fn handle_request(
    request: JsonRpcRequest,
    policy: &dyn ReadPolicy,
    write_context: Option<&McpWriteContext>,
) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse::err(request.id, -32600, "invalid jsonrpc version");
    }

    match request.method.as_str() {
        "tools/list" => JsonRpcResponse::ok(request.id, json!({ "tools": MCP_TOOLS })),
        "resources/list" => JsonRpcResponse::ok(
            request.id,
            json!({ "resources": list_authorized_resources(policy) }),
        ),
        "resources/read" => handle_resource_read(request, policy),
        "files/list" | "files/read" => handle_file_method(request, policy),
        "files.write" => handle_write_method(request, write_context),
        _ => JsonRpcResponse::err(request.id, -32601, "method not found"),
    }
}

fn path_param(params: &Value) -> Option<PathBuf> {
    params
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
}

fn handle_resource_read(request: JsonRpcRequest, policy: &dyn ReadPolicy) -> JsonRpcResponse {
    let Some(path) = path_param(&request.params) else {
        return JsonRpcResponse::err(request.id, -32602, "params.path is required");
    };

    match read_resource(&path, policy) {
        Ok(value) => JsonRpcResponse::ok(request.id, json!(value)),
        Err(err) => {
            log_mcp_denial_if_needed("resources/read", &path, &err.to_string());
            JsonRpcResponse::err(request.id, -32000, err.to_string())
        }
    }
}

fn handle_file_method(request: JsonRpcRequest, policy: &dyn ReadPolicy) -> JsonRpcResponse {
    let Some(path) = path_param(&request.params) else {
        return JsonRpcResponse::err(request.id, -32602, "params.path is required");
    };

    let result = match request.method.as_str() {
        "files/list" => list_files(&path, policy).map(|value| json!({ "entries": value })),
        "files/read" => read_file(&path, policy).map(|value| json!(value)),
        _ => unreachable!(),
    };

    match result {
        Ok(value) => JsonRpcResponse::ok(request.id, value),
        Err(err) => {
            log_mcp_denial_if_needed(&request.method, &path, &err.to_string());
            JsonRpcResponse::err(request.id, -32000, err.to_string())
        }
    }
}

fn handle_write_method(
    request: JsonRpcRequest,
    write_context: Option<&McpWriteContext>,
) -> JsonRpcResponse {
    let Some(context) = write_context else {
        return JsonRpcResponse::err(request.id, -32000, "write context is not configured");
    };

    match handle_files_write(
        &request.params,
        &context.permission_policy,
        &context.approval_store,
        &context.write_log_store,
    ) {
        Ok(WriteToolResult::ApprovalRequired(value)) => JsonRpcResponse::ok(request.id, value),
        Ok(WriteToolResult::Written(value)) => JsonRpcResponse::ok(request.id, value),
        Err(err) => JsonRpcResponse::err(request.id, -32000, err.to_string()),
    }
}

fn log_mcp_denial_if_needed(method: &str, path: &Path, message: &str) {
    if message.contains("permission denied") {
        warn!(
            "mcp access denied: method={} path={} reason={}",
            method,
            path.display(),
            message
        );
    }
}
