use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use provider::Database;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use transaction_pool::Pool;

use crate::{error::NetworkStartError, peer::PeerList, NetworkHandle, NetworkHandleMessage, NetworkManager, NoopImporter};

pub struct NetworkBuilder;

impl NetworkBuilder {
    pub async fn start_network<DB: Database + Send + Sync + 'static>(pool: Pool<DB>, cfg: NetworkConfig) -> Result<NetworkHandle, NetworkStartError> {
        // Server Binding
        let listener = match TcpListener::bind((cfg.address, cfg.port)).await {
            Ok(listner) => listner,
            Err(err) => return Err(NetworkStartError::LinstenerBindingError(err)),
        };
        let (tx, rx) = mpsc::unbounded_channel::<NetworkHandleMessage>();
        let rx_stream = UnboundedReceiverStream::new(rx); 
        let network_handle = NetworkHandle::new(tx);

        let network_manager = NetworkManager {
            listener,
            handle: network_handle.clone(),
            from_handle_rx: rx_stream,
            pool,
            peers: PeerList::new(),
            consensus: Box::new(NoopImporter),
            config: cfg.clone(),
        };

        // Finding peer from Boot Node
        // Initially, I implemented function that connects only boot node and never fails.
        network_manager.connect_with_boot_node(cfg.boot_node).await;
        // Network loop Start
        network_manager.start_loop();

        Ok(network_handle)
    }
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub address: IpAddr,
    pub port: u16,
    pub rpc_port: u16,
    pub max_peer_size: usize,
    pub boot_node: BootNode,
}

impl NetworkConfig {
    pub fn new(address: IpAddr, port: u16, rpc_port: u16) -> Self {
        Self {
            address,
            port,
            rpc_port,
            max_peer_size: 2,
            boot_node: BootNode::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BootNode {
    address: IpAddr,
    port: u16,
}

impl BootNode {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

impl Default for BootNode {
    fn default() -> Self {
        Self { address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port: 4321 }
    }
}
