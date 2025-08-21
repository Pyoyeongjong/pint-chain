use provider::{Database, ProviderFactory};

use crate::configs::{BlockConfig, ExecConfig, NetworkConfig, PoolConfig, RpcConfig};

pub struct LaunchContext {
    block_config: BlockConfig,
    pool_config: PoolConfig,
    network_config: NetworkConfig,
    rpc_config: RpcConfig,
    exec_config: ExecConfig,
}

pub struct NodeBuilder<DB: Database> {
    ctx: LaunchContext,
    provider: ProviderFactory<DB>,
}