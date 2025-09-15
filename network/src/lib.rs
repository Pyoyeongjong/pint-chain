use std::net::{IpAddr, SocketAddr};

use primitives::{handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage}, types::BlockHash};
use provider::{DatabaseTrait, ProviderFactory};
use tokio::{net::{TcpListener, TcpStream}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use transaction_pool::{identifier::TransactionOrigin, Pool};

use crate::{builder::{BootNode, NetworkConfig}, handle::NetworkHandle, peer::PeerList};

pub mod peer;
pub mod builder;
pub mod error;
pub mod handle;

pub struct NetworkManager<DB: DatabaseTrait> {
    listener: TcpListener,
    pub provider: ProviderFactory<DB>,
    networ_handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    peers: PeerList,
    consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    config: NetworkConfig,
}

impl<DB: DatabaseTrait + Sync + Send + 'static> NetworkManager<DB> {
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

                            NetworkHandleMessage::RequestDataResponse(from, address, port) => {
                                println!("#Network# RequestDataResponse is occured by {} {}", address, port);
                                let socket_addr = SocketAddr::from((address, port));
                                if let Some(peer) = this.peers.inner().read().iter().find(|peer| {
                                    *peer.addr() == socket_addr
                                }) {
                                    // send block datas
                                    let latest = this.provider.db().latest_block_number();
                                    for i in from..latest+1 {
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
                            NetworkHandleMessage::RequestData(from) => {
                                if this.peers.len() == 0 {
                                    println!("#Network# Can't find peer.");
                                    continue;
                                }
                                let peer = &this.peers.inner().read()[0];

                                peer.send(NetworkHandleMessage::RequestDataResponse(from,this.config.address, this.config.port));
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
                                    this.networ_handle.send(NetworkHandleMessage::RequestData(1));
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
                            NetworkHandleMessage::ReorgChainData => {
                                if this.peers.len() == 0 {
                                    println!("#Network# Can't find peer.");
                                    continue;
                                }
                                let peer = &this.peers.inner().read()[0];
                                peer.send(NetworkHandleMessage::RequestChainData(this.config.address, this.config.port));
                            }

                            NetworkHandleMessage::RequestChainData(ip_addr, port) => {
                                let socket_addr = SocketAddr::from((ip_addr, port));
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

                                let latest_bno = this.provider.db().latest_block_number();
                                let mut block_hash_vec: Vec<BlockHash> = Vec::new();
                                let start_bno = if latest_bno >= 16 {
                                    latest_bno - 16
                                } else {
                                    0
                                };

                                for i in start_bno..latest_bno {
                                    match this.provider.db().get_header(i) {
                                        Ok(header) => {
                                            block_hash_vec.push(header.calculate_hash());
                                        }
                                        Err(e) => {
                                            eprintln!("#Network# RequestChainData: Can't get block hash: {:?}", e);
                                            break;
                                        }
                                    }
                                }

                                peer.send(NetworkHandleMessage::RespondChainDataResult(block_hash_vec.len() as u64, block_hash_vec));
                            }

                            NetworkHandleMessage::RespondChainDataResult(_len, hash_vec) => {
                                let mut found = false;
                                for hash in hash_vec.iter().rev() {
                                    match this.provider.db().get_block_by_hash(hash.clone()) {
                                        Ok(block) => {
                                            found = true;
                                            let height = block.header().height;
                                            // delete datas
                                            if let Err(e) = this.provider.db().remove_datas(height) {
                                                eprintln!("#Network# RequestChainData: Failed to clean db datas {:?}", e);
                                                break;
                                            }
                                            // then request new data
                                            this.networ_handle.send(NetworkHandleMessage::RequestData(height+1));
                                            break;
                                        }
                                        Err(_e) => {
                                            continue;
                                        }
                                    }
                                }

                                // reorg chain from scratch
                                if !found {
                                    if let Err(e) = this.provider.db().remove_datas(0) {
                                        eprintln!("#Network# RequestChainData: Failed to clean db datas {:?}", e);
                                        break;
                                    }
                                    // then request new data
                                    this.networ_handle.send(NetworkHandleMessage::RequestData(1));
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

impl<DB: DatabaseTrait + std::fmt::Debug> std::fmt::Debug for NetworkManager<DB> {
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
