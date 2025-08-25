use std::{net::SocketAddr, sync::Arc, vec};

use primitives::{block::{Block, BlockImportable}, error::BlockImportError, transaction::SignedTransaction};
use provider::Database;
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use transaction_pool::{identifier::TransactionOrigin, Pool};

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
                        let peer_len = this.peers.len();
                        if peer_len >= this.config.max_peer_size {
                            println!("Can't accept a new peer. max_peer_size: {}", this.config.max_peer_size);
                        } else {
                            println!("New peer: {}", addr);
                            this.peers.insert_new_peer(socket, addr, this.handle.clone());
                        }
                    }

                    // NetworkHandle Message
                    Some(msg) = this.from_handle_rx.next() => {
                        println!("Network received message: {:?}", msg);
                        match msg {
                            NetworkHandleMessage::PeerConnectionTest{peer: addr} => {
                                let peer = match this.peers.find_peer(addr) {
                                    Some(peer) => peer,
                                    None => {
                                        eprintln!("Can't find this peer. {:?}", addr);
                                        continue;
                                    }
                                };
                                peer.send(NetworkHandleMessage::PeerConnectionTest { peer: addr });
                            }
                            NetworkHandleMessage::NewTransaction(signed) => {
                                let origin = TransactionOrigin::External;
                                let recovered = match signed.clone().into_recovered() {
                                    Ok(recovered) => recovered,
                                    Err(e) => {
                                        eprintln!("NewTransaction Recover Error: {:?}", e);
                                        continue;
                                    }
                                };
                                let res = this.pool.add_transaction(origin, recovered);

                                match res {
                                    Ok(_tx_hash) => {
                                        dbg!(_tx_hash);
                                        // broadcast to peer
                                        for peer in this.peers.inner().read().iter() {
                                            peer.send(NetworkHandleMessage::NewTransaction(signed.clone()));
                                        }
                                    }
                                    Err(pool_error) => {
                                        dbg!(pool_error.clone());
                                        eprintln!("NetTransaction Pool Error: {:?}", pool_error);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn connect_with_boot_node(&self, boot_node: BootNode) {
        if boot_node.is_boot_node() {
            return;
        }

        let addr: SocketAddr = boot_node.socket_addr();
        match TcpStream::connect(addr).await {
            Ok(socket) => {
                println!("Connected to boot node: {}", addr);
                self.peers.insert_new_peer(socket, addr, self.handle.clone());
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
    NewTransaction(SignedTransaction),
}

impl NetworkHandleMessage {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::PeerConnectionTest { peer: _ } => {
                let msg_type = 0x00 as u8;
                let payload_length = 0x00 as u8;
                let protocol_version = 0x00 as u8;

                let raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw
            }
            Self::NewTransaction(signed) => {
                let msg_type = 0x01 as u8;
                let payload_length = 0x41 as u8; // 65
                let protocol_version = 0x00 as u8;
                let mut data = signed.encode();

                let mut raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw.append(&mut data);
                raw
            }
        }
    }

    // First Byte: Message Type
    // Second Byte: Payload Length
    // Third Byte: Protocol Version
    // remains: Data
    pub fn decode(buf: &[u8], addr: SocketAddr) -> Option<NetworkHandleMessage>{
        if buf.len() < 3 {
            return None;
        }

        let msg_type = buf[0];
        let payload_length = buf[1] as usize;
        let protocol_version = buf[2];

        if buf.len() < 3 + payload_length {
            return None;
        }

        if protocol_version > 0 {
            return None;
        }

        let data = &buf[3..];

        match msg_type {
            0x00 => Some(NetworkHandleMessage::PeerConnectionTest{peer: addr}),
            0x01 => {
                let signed = match SignedTransaction::decode(&data.to_vec()) {
                    Ok((signed,_)) => signed,
                    Err(e) => {
                        return None
                    }
                };
                Some(NetworkHandleMessage::NewTransaction(signed))
            }
            _ => {
                None
            }
        }
    }
}
