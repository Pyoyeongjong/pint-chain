use std::net::{IpAddr, SocketAddr};

use primitives::handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage};
use provider::{Database, ProviderFactory};
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
    pub provider: ProviderFactory<DB>,
    networ_handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    peers: PeerList,
    consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    config: NetworkConfig,
}

impl<DB: Database + Sync + Send + 'static> NetworkManager<DB> {
    fn start_loop(self, is_boot_node: bool) {
        tokio::spawn(async move {
            println!("Network channel starts.");
            let mut this: NetworkManager<DB> = self;


            loop {
                
                tokio::select! {
                    // New Peer
                    Ok((socket, addr)) = this.listener.accept() => {
                        let peer_len = this.peers.len();
                        if peer_len >= this.config.max_peer_size {
                            println!("#Network# Can't accept a new peer. max_peer_size: {}", this.config.max_peer_size);
                        } else {
                            println!("New peer: {}", addr);
                            let (peer, pid) = this.peers.insert_new_peer(socket, addr, this.networ_handle.clone());
                            peer.send(NetworkHandleMessage::Hello(pid, this.config.address, this.config.port));
                            println!("####DBG####: Send Hello");
                        }
                    }

                    // NetworkHandle Message
                    Some(msg) = this.from_handle_rx.next() => {
                        println!("#Network# received message: {:?}", msg);
                        match msg {
                            NetworkHandleMessage::PeerConnectionTest{peer: addr} => {
                                let peer = match this.peers.find_peer(addr) {
                                    Some(peer) => peer,
                                    None => {
                                        eprintln!("#Network# Can't find this peer. {:?}", addr);
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
                                        eprintln!("#Network# NewTransaction Recover Error: {:?}", e);
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
                                        eprintln!("#Network# New Transaction Pool Error: {:?}", pool_error);
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

                            NetworkHandleMessage::RequestDataResponseFinished => {
                                println!("#Network# Finished Syncronizing");
                            }

                            NetworkHandleMessage::RequestDataResponse(address, port) => {
                                println!("#Network# RequestDataResponse is occured by {} {}", address, port);
                                let socket_addr = SocketAddr::from((address, port));
                                if let Some(peer) = this.peers.inner().read().iter().find(|peer| {
                                    *peer.addr() == socket_addr
                                }) {
                                    // send block datas
                                    let latest = this.provider.db().latest_block_number();
                                    for i in 1..latest+1 {
                                        match this.provider.db().get_block(i) {
                                            Ok(block) => peer.send(NetworkHandleMessage::NewPayload(block)),
                                            Err(e) => {
                                                eprintln!("#Network# Failed to get block in db: {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                    println!("#Network# Block Sync Ok! {} {}", address, port);
                                    // peer.send(NetworkHandleMessage::RequestDataResponseFinished);
                                } else {
                                    println!("#Network# Can't find peer! {} {}", address, port);
                                }

                            }

                            // request db, pool data to
                            NetworkHandleMessage::RequestData => {
                                if this.peers.len() == 0 {
                                    println!("#Network# Can't find peer.");
                                    continue;
                                }
                                let peer = &this.peers.inner().read()[0];

                                peer.send(NetworkHandleMessage::RequestDataResponse(this.config.address, this.config.port));
                                println!("#Network# Requested Data.");
                            }

                            NetworkHandleMessage::HandShake(pid, address, port) => {
                                let socket_addr = SocketAddr::from((address, port));
                                let mut binding = this.peers.inner().write();
                                let peer = match binding.iter_mut().find(|peer| {

                                    peer.id() == pid
                                }) {
                                    Some(peer) => peer,
                                    None => {
                                        eprintln!("#Network# Handshake: Can't find peer");
                                        continue;
                                    }
                                };
                                peer.update_addr(socket_addr);
                
                                println!("#Network# Handshake completed with {:?}", socket_addr);
                                
                            }

                            NetworkHandleMessage::Hello(pid, address, port) => {
                                let socket_addr = SocketAddr::from((address, port));
                                let binding = this.peers.inner().read();
                                let peer = match binding.iter().find(|peer| {
                                    *peer.addr() == socket_addr
                                }) {
                                    Some(peer) => peer,
                                    None => {
                                        eprintln!("#Network# Hello: Can't find peer");
                                        continue;
                                    }
                                };
                                peer.send(NetworkHandleMessage::HandShake(pid, this.config.address, this.config.port));
                                println!("####DBG####: Send HandShake");

                                if !is_boot_node {
                                    println!("#Network# Try to synchronize db and mem-pool.");
                                    this.networ_handle.send(NetworkHandleMessage::RequestData);
                                }
                            }

                            NetworkHandleMessage::RemovePeer(pid) => {
                                this.peers.remove_peer_by_id(pid);
                            }

                            NetworkHandleMessage::BroadcastTransaction(signed) => {
                                for peer in this.peers.inner().read().iter() {
                                    peer.send(NetworkHandleMessage::NewTransaction(signed.clone()));
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn connect_with_boot_node(&mut self, _ip_addr: IpAddr, _port: u16, boot_node: &BootNode) {
        if boot_node.is_boot_node() {
            return;
        }

        let addr: SocketAddr = boot_node.socket_addr();
        match TcpStream::connect(addr).await {
            Ok(socket) => {
                println!("Connected to boot node: {}", addr);
                let _ = self.peers.insert_new_peer(socket, addr, self.networ_handle.clone());
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
