use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::{
    list_authorized_resources, list_files, read_file, read_resource, ReadPolicy,
};
use super::write_tools::{handle_files_write, WriteToolResult};
use crate::product::approvals::store::ApprovalStore;
use crate::product::approvals::write_log::WriteLogStore;
use crate::product::logs::audit::append_audit_log;
use crate::product::logs::event::LogLevel;
use crate::product::logs::store::AuditLogStore;
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
    pub audit_log_store: AuditLogStore,
}

pub fn handle_request(
    request: JsonRpcRequest,
    policy: &dyn ReadPolicy,
    write_context: Option<&McpWriteContext>,
) -> JsonRpcResponse {
    let request_id = request_id_from_jsonrpc_id(&request.id);
    let method = request.method.clone();
    let audit = write_context.map(|context| &context.audit_log_store);

    if let Some(store) = audit {
        append_audit_log(
            store,
            Some(request_id.clone()),
            "mcp.request",
            LogLevel::Info,
            "MCP request received",
            json!({
                "method": method,
                "params": safe_params_for_log(&request.method, &request.params),
            }),
        );
    }

    let response = if request.jsonrpc != "2.0" {
        JsonRpcResponse::err(request.id, -32600, "invalid jsonrpc version")
    } else {
        match request.method.as_str() {
            "tools/list" => JsonRpcResponse::ok(request.id, json!({ "tools": MCP_TOOLS })),
            "resources/list" => JsonRpcResponse::ok(
                request.id,
                json!({ "resources": list_authorized_resources(policy) }),
            ),
            "resources/read" => handle_resource_read(request, policy, audit, &request_id),
            "files/list" | "files/read" => handle_file_method(request, policy, audit, &request_id),
            "files.write" => handle_write_method(request, write_context, &request_id),
            _ => JsonRpcResponse::err(request.id, -32601, "method not found"),
        }
    };

    if let Some(store) = audit {
        if let Some(error) = response.error.as_ref() {
            append_audit_log(
                store,
                Some(request_id),
                "mcp.error",
                LogLevel::Error,
                "MCP request failed",
                json!({
                    "method": method,
                    "code": error.code,
                    "error": error.message,
                }),
            );
        } else {
            append_audit_log(
                store,
                Some(request_id),
                "mcp.response",
                LogLevel::Info,
                "MCP request completed",
                json!({ "method": method }),
            );
        }
    }

    response
}

fn request_id_from_jsonrpc_id(id: &Value) -> String {
    match id {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn safe_params_for_log(method: &str, params: &Value) -> Value {
    if method == "files.write" {
        return json!({
            "path": params.get("path").and_then(Value::as_str),
            "contentBytes": params
                .get("content")
                .and_then(Value::as_str)
                .map(|value| value.len()),
            "hasApprovalId": params.get("approvalId").is_some(),
            "hasDiff": params.get("diff").is_some(),
        });
    }

    params.clone()
}

fn path_param(params: &Value) -> Option<PathBuf> {
    params
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
}

fn handle_resource_read(
    request: JsonRpcRequest,
    policy: &dyn ReadPolicy,
    audit: Option<&AuditLogStore>,
    request_id: &str,
) -> JsonRpcResponse {
    let Some(path) = path_param(&request.params) else {
        return JsonRpcResponse::err(request.id, -32602, "params.path is required");
    };

    match read_resource(&path, policy) {
        Ok(value) => {
            if let Some(store) = audit {
                append_audit_log(
                    store,
                    Some(request_id.to_string()),
                    "file.read",
                    LogLevel::Info,
                    "resource descriptor read",
                    json!({ "method": "resources/read", "path": path }),
                );
            }

            JsonRpcResponse::ok(request.id, json!(value))
        }
        Err(err) => {
            log_mcp_denial_if_needed("resources/read", &path, &err.to_string());

            if let Some(store) = audit {
                append_audit_log(
                    store,
                    Some(request_id.to_string()),
                    "permission.deny",
                    LogLevel::Warn,
                    "resource read denied",
                    json!({
                        "method": "resources/read",
                        "path": path,
                        "error": err.to_string(),
                    }),
                );
            }

            JsonRpcResponse::err(request.id, -32000, err.to_string())
        }
    }
}

fn handle_file_method(
    request: JsonRpcRequest,
    policy: &dyn ReadPolicy,
    audit: Option<&AuditLogStore>,
    request_id: &str,
) -> JsonRpcResponse {
    let Some(path) = path_param(&request.params) else {
        return JsonRpcResponse::err(request.id, -32602, "params.path is required");
    };

    let method = request.method.clone();

    let result = match request.method.as_str() {
        "files/list" => list_files(&path, policy).map(|value| json!({ "entries": value })),
        "files/read" => read_file(&path, policy).map(|value| json!(value)),
        _ => unreachable!(),
    };

    match result {
        Ok(value) => {
            if let Some(store) = audit {
                append_audit_log(
                    store,
                    Some(request_id.to_string()),
                    if method == "files/read" {
                        "file.read"
                    } else {
                        "mcp.response"
                    },
                    LogLevel::Info,
                    if method == "files/read" {
                        "file read through MCP"
                    } else {
                        "file list through MCP"
                    },
                    json!({
                        "method": method,
                        "path": path,
                    }),
                );
            }

            JsonRpcResponse::ok(request.id, value)
        }
        Err(err) => {
            log_mcp_denial_if_needed(&request.method, &path, &err.to_string());

            if let Some(store) = audit {
                append_audit_log(
                    store,
                    Some(request_id.to_string()),
                    "permission.deny",
                    LogLevel::Warn,
                    "file access denied",
                    json!({
                        "method": request.method,
                        "path": path,
                        "error": err.to_string(),
                    }),
                );
            }

            JsonRpcResponse::err(request.id, -32000, err.to_string())
        }
    }
}

fn handle_write_method(
    request: JsonRpcRequest,
    write_context: Option<&McpWriteContext>,
    request_id: &str,
) -> JsonRpcResponse {
    let Some(context) = write_context else {
        return JsonRpcResponse::err(request.id, -32000, "write context is not configured");
    };

    match handle_files_write(
        Some(request_id.to_string()),
        &request.params,
        &context.permission_policy,
        &context.approval_store,
        &context.write_log_store,
        &context.audit_log_store,
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
