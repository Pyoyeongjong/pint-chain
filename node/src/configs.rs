use std::net::Ipv4Addr;

pub struct PoolConfig {}
pub struct NetworkConfig {
    ip_addr: Ipv4Addr,
    port: u16,
    rpc_port: u16,
    max_peer_size: u16,
}
pub struct RpcConfig {}
pub struct BlockConfig {}
pub struct ExecConfig {}
