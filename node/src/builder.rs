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
    pub fn new(network_config: NetworkConfig) -> Self {
        Self {
            block_config: BlockConfig::default(),
            pool_config: PoolConfig::default(),
            network_config,
            rpc_config: RpcConfig::default(),
            exec_config: ExecConfig::default(),
        }
    }
}

impl LaunchContext {
    pub async fn launch(self) -> Result<Node<Arc<InMemoryDB>>, NodeLaunchError> {
        let Self {network_config, ..} = self;
        // Build Provider
        let db = Arc::new(InMemoryDB::new());
        let provider = ProviderFactory::new(db);
        // Build Pool
        let pool = Pool::new(provider.clone());
        // Build PayloadBuilder
        let builder = PayloadBuilder::new(provider.clone(), pool.clone());
        // Build Network
        let network_handle = NetworkBuilder::start_network(pool.clone(), network_config).await?;
        // Build Miner
        let (miner_handle, consensus_rx) = Miner::build_miner();
        // Build Consensus
        let consensus = ConsensusEngine::new(
            pool.clone(), 
            builder.clone(), 
            network_handle.clone(), 
            provider.clone(), 
            miner_handle, 
            consensus_rx
        );

        Ok(Node {
            provider,
            pool,
            builder,
            consensus,
            network: network_handle,
        })
    }
}