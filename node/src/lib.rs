use axum::{routing::{post}, Router};
use consensus::{miner::Miner, ConsensusEngine};
use network::{builder::NetworkConfig, NetworkHandle, NetworkManager};
use payload::PayloadBuilder;
use provider::{Database, ProviderFactory};
use tokio::net::TcpListener;
use transaction_pool::Pool;

use crate::rpc::rpc_handle;

pub mod builder;
pub mod configs;
pub mod error;
pub mod rpc;

#[derive(Debug)]
pub struct Node<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
    builder: PayloadBuilder<DB>,
    consensus: ConsensusEngine<DB>,
    network: NetworkHandle,
}

impl<DB: Database> Node<DB> {
    pub async fn run_rpc(&self, network_config: NetworkConfig) -> Result<(), Box<dyn std::error::Error>> {

        let listener = match TcpListener::bind((network_config.address, network_config.rpc_port)).await {
            Ok(listener) => listener,
            Err(e) => return Err(Box::new(e)),
        };

        let app = Router::new().route("/", post(rpc_handle));

        let _ = match axum::serve(listener, app).await {
            Ok(_) => {},
            Err(e) => return Err(Box::new(e)),
        };

        Ok(())

    }
}