use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use consensus::ConsensusEngine;
use network::{builder::NetworkConfig};
use primitives::{handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage}, transaction::SignedTransaction};
use provider::{Database, ProviderFactory};
use serde_json::json;
use tokio::net::TcpListener;
use transaction_pool::{identifier::TransactionOrigin, Pool};

use crate::rpc::{RpcRequest, RpcResponse};

pub mod builder;
pub mod configs;
pub mod error;
pub mod rpc;

#[derive(Debug)]
pub struct Node<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
    consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    network: Box<dyn Handle<Msg = NetworkHandleMessage>>,
}

impl<DB: Database> Node<DB> {
    pub async fn run_rpc(self, network_config: NetworkConfig) -> Result<(), Box<dyn std::error::Error>> {

        println!("PintCnain Node Rpc Server starts.");

        let listener = match TcpListener::bind((network_config.address, network_config.rpc_port)).await {
            Ok(listener) => listener,
            Err(e) => return Err(Box::new(e)),
        };

        let node = Arc::new(self);

        let app = Router::new()
            .route("/", post(rpc_handle::<DB>))
            .with_state(node);

        let _ = match axum::serve(listener, app).await {
            Ok(_) => {},
            Err(e) => return Err(Box::new(e)),
        };

        Ok(())

    }
}

pub async fn rpc_handle<DB: Database>(State(node): State<Arc<Node<DB>>>, Json(req): Json<RpcRequest>) -> Json<RpcResponse> {
    let mut result = json!("method not found");
    let mut success = false;
    dbg!(&req.params[0].as_str());    match req.method.as_str() {
        "chain_name" => {
            result = json!("Pint");
        }
        "local_transaction" => {
            if let Some(raw) = req.params[0].as_str() {
                dbg!(raw);
                let data = match hex::decode(raw) {
                    Ok(data) => data,
                    Err(e) => return Json(RpcResponse { jsonrpc: "2.0".to_string(), success, result: json!("Transaction Hex Decode Error"), id: req.id })
                };
                let signed = match SignedTransaction::decode(&data) {
                    Ok((signed, _)) => signed,
                    Err(e) => return Json(RpcResponse { jsonrpc: "2.0".to_string(), success, result: json!("Transaction Decode Error"), id: req.id })
                };
                let origin = TransactionOrigin::Local;
                let recovered = match signed.into_recovered() {
                    Ok(recovered) => recovered,
                    Err(e) => return Json(RpcResponse { jsonrpc: "2.0".to_string(), success, result: json!("Transaction Recovery Error"), id: req.id })
                };
                let tx_hash = match node.pool.add_transaction(origin, recovered) {
                    Ok(tx_hash) => tx_hash,
                    Err(e) => return Json(RpcResponse { jsonrpc: "2.0".to_string(), success, result: json!("Transaction Pool Error"), id: req.id })
                };

                // broadcast to peer!

                success = true;
                result = json!(tx_hash.to_string());
            } else {
                result = json!("There is no new transaction");
            }
        }
        _ => {
            result = json!("Wrong Requirement");
        }
    }

    Json(RpcResponse { jsonrpc: "2.0".to_string(), success, result, id: req.id })
}