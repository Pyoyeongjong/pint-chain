use std::sync::Arc;

use consensus::{miner::Miner, ConsensusEngine};
use database::db::InMemoryDB;
use network::builder::{NetworkBuilder, NetworkConfig};
use payload::PayloadBuilder;
use provider::ProviderFactory;
use transaction_pool::Pool;

use crate::{configs::{BlockConfig, ExecConfig, PoolConfig, RpcConfig}, error::NodeLaunchError, Node};

pub struct LaunchContext {
    block_config: BlockConfig,
    pool_config: PoolConfig,
    network_config: NetworkConfig,
    rpc_config: RpcConfig,
    exec_config: ExecConfig,
}

impl LaunchContext {
    pub fn new(network_config: NetworkConfig, block_config: BlockConfig) -> Self {
        Self {
            block_config,
            pool_config: PoolConfig::default(),
            network_config,
            rpc_config: RpcConfig::default(),
            exec_config: ExecConfig::default(),
        }
    }
}

impl LaunchContext {
    pub async fn launch(self) -> Result<Node<Arc<InMemoryDB>>, NodeLaunchError> {
        let Self {network_config, block_config,..} = self;
        // Build Provider
        let db = Arc::new(InMemoryDB::genesis_block());
        let provider = ProviderFactory::new(db);
        // Build Pool
        let pool = Pool::new(provider.clone());
        // Build PayloadBuilder
        let builder = PayloadBuilder::new(block_config.miner_address, provider.clone(), pool.clone());
        let (builder_handle, builder_rx) = builder.start_builder();
        // Build Network
        let network_handle = NetworkBuilder::start_network(pool.clone(), network_config).await?;
        // Build Miner
        let (miner_handle, miner_rx) = Miner::build_miner();
        // Build Consensus
        let consensus = ConsensusEngine::new(
            pool.clone(), 
            builder_handle,
            Box::new(network_handle.clone()), 
            provider.clone(), 
            miner_handle, 
            miner_rx,
            builder_rx,
        );

        let consensus_handle = consensus.start_consensus();

        Ok(Node {
            provider,
            pool,
            consensus: Box::new(consensus_handle),
            network: Box::new(network_handle),
        })
    }
}