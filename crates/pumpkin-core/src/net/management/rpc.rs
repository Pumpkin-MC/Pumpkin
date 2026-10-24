use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSON_RPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl RpcError {
    #[must_use]
    pub fn parse_error(data: Option<String>) -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data,
        }
    }

    #[must_use]
    pub fn invalid_request(data: Option<String>) -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data,
        }
    }

    #[must_use]
    pub fn method_not_found(data: Option<String>) -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data,
        }
    }

    #[must_use]
    pub fn invalid_params(data: Option<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data,
        }
    }

    #[must_use]
    pub fn internal_error(data: Option<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    #[must_use]
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn error(id: Value, error: RpcError) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl RpcNotification {
    pub fn new(method: impl Into<String>, param: Option<Value>) -> Self {
        let params = param.map(|p| Value::Array(vec![p]));
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }
}

#[must_use]
pub fn is_valid_request_id(id: &Value) -> bool {
    id.is_null() || id.is_number() || id.is_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_request_ids() {
        assert!(is_valid_request_id(&json!(null)));
        assert!(is_valid_request_id(&json!(1)));
        assert!(is_valid_request_id(&json!("req-123")));

        assert!(!is_valid_request_id(&json!(true)));
        assert!(!is_valid_request_id(&json!([1, 2])));
        assert!(!is_valid_request_id(&json!({"id": 1})));
    }

    #[test]
    fn rpc_response_success_serialization() {
        let resp = RpcResponse::success(json!(1), json!({"status": "ok"}));
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains(r#""jsonrpc":"2.0""#));
        assert!(s.contains(r#""id":1"#));
        assert!(s.contains(r#""result":{"status":"ok"}"#));
        assert!(!s.contains("error"));
    }

    #[test]
    fn rpc_response_error_serialization() {
        let err = RpcError::method_not_found(Some("Method not recognized".to_string()));
        let resp = RpcResponse::error(json!("123"), err);
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains(r#""code":-32601"#));
        assert!(s.contains(r#""message":"Method not found""#));
        assert!(s.contains(r#""data":"Method not recognized""#));
        assert!(!s.contains("result"));
    }

    #[test]
    fn notification_wraps_param_in_array() {
        let notif = RpcNotification::new("minecraft:server/status", Some(json!({"started": true})));
        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, "minecraft:server/status");
        let params = notif.params.unwrap();
        assert!(params.is_array());
        assert_eq!(params.as_array().unwrap().len(), 1);
    }
}
