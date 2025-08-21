use consensus::ConsensusEngine;
use network::{NetworkHandle, NetworkManager};
use payload::PayloadBuilder;
use provider::{Database, ProviderFactory};
use transaction_pool::Pool;

pub mod builder;
pub mod configs;

pub struct Node<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
    builder: PayloadBuilder<DB>,
    consensus: ConsensusEngine<DB>,
    network: NetworkHandle,
    p2p: NetworkManager<DB>,
}