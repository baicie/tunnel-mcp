use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::{list_files, read_file, ReadPolicy};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const MCP_TOOLS: &[&str] = &[
    "resources/list",
    "resources/read",
    "files/list",
    "files/read",
];
pub const MCP_RESOURCES: &[&str] = &["filesystem"];

pub fn handle_request(request: JsonRpcRequest, policy: &dyn ReadPolicy) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse::err(request.id, -32600, "invalid jsonrpc version");
    }

    match request.method.as_str() {
        "tools/list" => JsonRpcResponse::ok(request.id, json!({ "tools": MCP_TOOLS })),
        "resources/list" => JsonRpcResponse::ok(request.id, json!({ "resources": MCP_RESOURCES })),
        "files/list" | "resources/read" | "files/read" => handle_file_method(request, policy),
        _ => JsonRpcResponse::err(request.id, -32601, "method not found"),
    }
}

fn handle_file_method(request: JsonRpcRequest, policy: &dyn ReadPolicy) -> JsonRpcResponse {
    let path = request
        .params
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from);
    let Some(path) = path else {
        return JsonRpcResponse::err(request.id, -32602, "params.path is required");
    };

    let result = match request.method.as_str() {
        "files/list" => list_files(&path, policy).map(|value| json!({ "entries": value })),
        "resources/read" | "files/read" => read_file(&path, policy).map(|value| json!(value)),
        _ => unreachable!(),
    };

    match result {
        Ok(value) => JsonRpcResponse::ok(request.id, value),
        Err(err) => JsonRpcResponse::err(request.id, -32000, err.to_string()),
    }
}
