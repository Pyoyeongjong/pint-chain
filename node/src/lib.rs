use std::sync::Arc;

use axum::{routing::post, Router};
use network::{builder::NetworkConfig};
use primitives::{handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage}, transaction::SignedTransaction};
use provider::{DatabaseTrait, ProviderFactory};
use tokio::net::TcpListener;
use transaction_pool::{identifier::TransactionOrigin, Pool};
use crate::rpc::rpc_handle;

pub mod builder;
pub mod configs;
pub mod error;
pub mod rpc;

#[derive(Debug)]
pub struct Node<DB: DatabaseTrait> {
    pub provider: ProviderFactory<DB>,
    pub pool: Pool<DB>,
    pub consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    pub network: Box<dyn Handle<Msg = NetworkHandleMessage>>,
}

impl<DB: DatabaseTrait> Node<DB> {

    pub fn handle_tx(&self, tx: SignedTransaction) {
        let tx_hash = tx.hash;
        let recovered = match tx.into_recovered() {
            Ok(recovered) => {
                recovered
            }
            Err(_e) => {
                eprintln!("Failed to handle tx: {:?}", tx_hash);
                return;
            }
        };

        if let Err(_e) = self.pool.add_transaction(TransactionOrigin::External, recovered) {
            eprintln!("Failed to handle tx: {:?}", tx_hash);
        }
    }

    pub fn handle_consensus(&self, msg: ConsensusHandleMessage) {
        self.consensus.send(msg);
    }

    pub fn handle_network(&self, msg: NetworkHandleMessage) {
        self.network.send(msg);
    }

    pub async fn run_rpc(self, network_config: NetworkConfig) -> Result<(), Box<dyn std::error::Error>> {

        println!("[ RPC ] PintChain Node Rpc Server starts.");

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