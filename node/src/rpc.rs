use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
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
    pub jsonrpc: &'static str,
    pub result: Value,
    pub id: u64,
}

pub async fn rpc_handle(Json(req): Json<RpcRequest>) -> Json<RpcResponse> {
    let mut result = json!("method not found");

    match req.method.as_str() {
        "chain_name" => {
            result = json!("Pint");
        }
        _ => {}
    }

    Json(RpcResponse { jsonrpc: "2.0", result, id: req.id })
}