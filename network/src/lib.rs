use std::{net::SocketAddr};

use primitives::{block::{Block, BlockValidationResult}, error::BlockImportError, handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage}};
use provider::Database;
use tokio::{net::{TcpListener, TcpStream}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use transaction_pool::{identifier::TransactionOrigin, Pool};

use crate::{builder::{BootNode, NetworkConfig}, handle::NetworkHandle, peer::PeerList};

pub mod peer;
pub mod builder;
pub mod error;
pub mod handle;

pub struct NetworkManager<DB: Database> {
    listener: TcpListener,
    networ_handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    peers: PeerList,
    consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    config: NetworkConfig,
}

impl<DB: Database + Sync + Send + 'static> NetworkManager<DB> {
    fn start_loop(self) {
        tokio::spawn(async move {
            println!("Network channel starts.");
            let mut this = self;
            loop {
                tokio::select! {
                    // New Peer
                    Ok((socket, addr)) = this.listener.accept() => {
                        let peer_len = this.peers.len();
                        if peer_len >= this.config.max_peer_size {
                            println!("(Network) Can't accept a new peer. max_peer_size: {}", this.config.max_peer_size);
                        } else {
                            println!("New peer: {}", addr);
                            this.peers.insert_new_peer(socket, addr, this.networ_handle.clone());
                        }
                    }

                    // NetworkHandle Message
                    Some(msg) = this.from_handle_rx.next() => {
                        println!("(Network) Network received message: {:?}", msg);
                        match msg {
                            NetworkHandleMessage::PeerConnectionTest{peer: addr} => {
                                let peer = match this.peers.find_peer(addr) {
                                    Some(peer) => peer,
                                    None => {
                                        eprintln!("(Network) Can't find this peer. {:?}", addr);
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
                                        eprintln!("(Network) NewTransaction Recover Error: {:?}", e);
                                        continue;
                                    }
                                };
                                let res = this.pool.add_transaction(origin, recovered);

                                match res {
                                    Ok(_tx_hash) => {
                                        // broadcast to peer
                                        for peer in this.peers.inner().read().iter() {
                                            peer.send(NetworkHandleMessage::NewTransaction(signed.clone()));
                                        }
                                    }
                                    Err(pool_error) => {
                                        eprintln!("(Network) New Transaction Pool Error: {:?}", pool_error);
                                        continue;
                                    }
                                }
                            }
                            NetworkHandleMessage::NewPayload(block) => {
                                this.consensus.send(ConsensusHandleMessage::ImportBlock(block));

                            }

                            NetworkHandleMessage::BroadcastBlock(block) => {
                                for peer in this.peers.inner().read().iter() {
                                    peer.send(NetworkHandleMessage::NewPayload(block.clone()));
                                }
                            }

                            NetworkHandleMessage::UpdateData => {
                                todo!()
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
                self.peers.insert_new_peer(socket, addr, self.networ_handle.clone());
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
            .field("handle", &self.networ_handle)
            .field("from_handle_rx", &self.from_handle_rx)
            .field("pool", &self.pool)
            .field("peers", &self.peers)
            .field("config", &self.config).finish()
    }
}


#[derive(Debug)]
pub struct NoopConsensusHandle;

impl Handle for NoopConsensusHandle {
    type Msg = ConsensusHandleMessage;
    fn send(&self, _block: Self::Msg) {
        // Do nothing
    }
}
