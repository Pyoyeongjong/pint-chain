use serde::{Deserialize, Serialize};
use serde_json::{Value};

#[derive(Debug, Deserialize, Clone)]
pub struct RpcRequest{
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<Value>,
    pub id: u64
}

impl RpcRequest {
    pub fn noob() -> Self {
        Self {
            jsonrpc: "abc".to_string(),
            method: "test".to_string(),
            params: Vec::new(),
            id: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub success: bool,
    pub result: Value,
    pub id: u64,
}
