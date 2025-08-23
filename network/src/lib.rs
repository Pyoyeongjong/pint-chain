use std::{net::SocketAddr, sync::Arc};

use primitives::{block::{Block, BlockImportable}, error::BlockImportError};
use provider::Database;
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use transaction_pool::Pool;

use crate::{builder::{BootNode, NetworkConfig}, peer::PeerList};

pub mod peer;
pub mod builder;
pub mod error;

pub struct NetworkManager<DB: Database> {
    listener: TcpListener,
    handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    peers: PeerList,
    consensus: Box<dyn BlockImportable<B = Block>>,
    config: NetworkConfig,
}

impl<DB: Database + Sync + Send + 'static> NetworkManager<DB> {
    fn start_loop(self) {
        tokio::spawn(async move {
            let mut this = self;
            loop {
                tokio::select! {
                    // New Peer
                    Ok((socket, addr)) = this.listener.accept() => {
                        if this.peers.len().await >= this.config.max_peer_size {
                            println!("Can't accept a new peer. max_peer_size: {}", this.config.max_peer_size);
                        } else {
                            println!("New peer: {}", addr);
                            this.peers.insert_new_peer(socket, addr, this.handle.clone()).await;
                        }
                        
                    }

                    // NetworkHandle Message
                    Some(msg) = this.from_handle_rx.next() => {
                        println!("Network received message: {:?}", msg);
                        match msg {
                            NetworkHandleMessage::PeerConnectionTest{peer: addr} => {
                                let peer = this.peers.find_peer(addr).await.unwrap();
                                peer.send(NetworkHandleMessage::PeerConnectionTest { peer: addr });
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn connect_with_boot_node(&self, boot_node: BootNode) {
        let addr: SocketAddr = boot_node.socket_addr();
        match TcpStream::connect(addr).await {
            Ok(socket) => {
                println!("Connected to boot node: {}", addr);
                self.peers.insert_new_peer(socket, addr, self.handle.clone()).await;
            }
            Err(e) => {
                eprintln!("Failed to connect to the boot node {}: {:?}", addr, e);
            }
        }
    }
}

impl<DB: Database + std::fmt::Debug> std::fmt::Debug for NetworkManager<DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkManager")
            .field("listener", &self.listener)
            .field("handle", &self.handle)
            .field("from_handle_rx", &self.from_handle_rx)
            .field("pool", &self.pool)
            .field("peers", &self.peers)
            .field("config", &self.config).finish()
    }
}

#[derive(Clone, Debug)]
pub struct NetworkHandle {
    inner: Arc<NetworkInner>,
}

impl NetworkHandle {
    pub fn new(tx: UnboundedSender<NetworkHandleMessage>) -> Self {
        Self {
            inner: Arc::new(NetworkInner{ to_manager_tx: tx})
        }
    }

    pub fn send(&self, msg: NetworkHandleMessage) {
        if let Err(e) = self.inner.to_manager_tx.send(msg) {
            eprintln!("Failed to send NetworkHandleMessage: {:?}", e);
        }
    }
}

#[derive(Debug)]
pub struct NetworkInner {
    to_manager_tx: UnboundedSender<NetworkHandleMessage>,
}

pub struct NoopImporter;

impl BlockImportable for NoopImporter {
    type B = Block;
    fn import_block(&self, _block: Self::B) -> Result<(), primitives::error::BlockImportError> {
        Err(BlockImportError::NoopImporter)
    }
}

#[derive(Debug)]
pub enum NetworkHandleMessage {
    PeerConnectionTest{
        peer: SocketAddr
    },
}

impl NetworkHandleMessage {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::PeerConnectionTest { peer: _ } => {
                b"Peer Connection Test.\n".to_vec()
            }
        }
    }

    pub fn decode(buf: &[u8]) -> Option<NetworkHandleMessage>{
        None
    }
}
