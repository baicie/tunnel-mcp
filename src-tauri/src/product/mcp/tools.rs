use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::{
    list_authorized_resources, list_files, read_file, read_resource, ReadPolicy,
};
use log::warn;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const MCP_TOOLS: &[&str] = &[
    "resources/list",
    "resources/read",
    "files/list",
    "files/read",
];

pub fn handle_request(request: JsonRpcRequest, policy: &dyn ReadPolicy) -> JsonRpcResponse {
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
