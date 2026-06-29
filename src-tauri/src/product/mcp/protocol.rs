use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonRpcRequest, JsonRpcResponse};
    use serde_json::{json, Value};

    #[test]
    fn ok_response_carries_result() {
        let response = JsonRpcResponse::ok(json!(1), json!({"value": 42}));
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(1));
        assert_eq!(response.result, Some(json!({"value": 42})));
        assert!(response.error.is_none());
    }

    #[test]
    fn err_response_carries_error() {
        let response = JsonRpcResponse::err(json!("req-1"), -32600, "bad request");
        assert_eq!(response.id, json!("req-1"));
        assert!(response.result.is_none());
        let err = response.error.expect("error");
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "bad request");
    }

    #[test]
    fn request_defaults_params_to_null() {
        let parsed: JsonRpcRequest = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
        }))
        .expect("parse request");
        assert_eq!(parsed.params, Value::Null);
    }
}
