use std::{net::SocketAddr, sync::{Arc}};

use parking_lot::RwLock;
use primitives::handle::Handle;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::{mpsc::{self, UnboundedSender}}};

use crate::{NetworkHandle, NetworkHandleMessage};

#[derive(Debug, Clone)]
pub struct Peer {
    addr: SocketAddr,
    tx: UnboundedSender<NetworkHandleMessage>,
}

impl Peer {
    pub fn new(addr: SocketAddr, tx: UnboundedSender<NetworkHandleMessage>) -> Self {
        Self { addr, tx }
    }

    pub fn send(&self, msg: NetworkHandleMessage) {
        if let Err(e) = self.tx.send(msg) {
            eprintln!("Failed to send NetworkHandleMessage: {:?}", e);
        }
    }
}


#[derive(Debug)]
pub struct PeerList {
    peers: Arc<RwLock<Vec<Peer>>>,
}

impl PeerList {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(Vec::new()))
        }
    }

    pub fn inner(&self) -> &RwLock<Vec<Peer>> {
        &self.peers
    }

    pub fn len(&self) -> usize {
        self.peers.read().len()
    }

    pub fn find_peer(&self, addr: SocketAddr) -> Option<Peer> {
        let peers = self.peers.read();
        for peer in peers.iter() {
            if peer.addr == addr {
                return Some(peer.clone());
            }
        }
        None
    }
}

impl PeerList {
    pub fn insert_new_peer(&self, socket: TcpStream, addr: SocketAddr, network_handle: NetworkHandle) {
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkHandleMessage>();
        // tx is used for every componets who want to send peer msg
        // rx isolates socket
        let new_peer = Peer::new(addr.clone(), tx);
        let mut peers = self.peers.write();
        peers.push(new_peer);

        let (mut read_socket, mut write_socket) = socket.into_split();

        // incoming loop
        let incoming = async move {
            println!("Peer {:?} incoming task has spawned.", addr);
            let mut buf = [0u8; 1024];
            loop {
                match read_socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Peer {:?} closed connection", addr);
                        break;
                    }
                    Ok(n) => {
                        if let Some(decoded) = NetworkHandleMessage::decode(&buf[..n], addr) {
                            let _ = network_handle.send(decoded);
                        } else {
                            eprintln!("Invalid Request from Peer: {:?}", addr);
                        }
                    }
                    Err(e) => {
                        eprintln!("read error from {:?}: {:?}", addr, e);
                        break;
                    }
                }
            }
        };

        // outgoing loop
        let outgoing = async move {
            println!("Peer {:?} outgoing task has spawned.", addr);
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write_socket.write_all(&msg.encode()).await {
                    eprintln!("Failed to send to {:?}: {:?}", addr, e);
                    break;
                }
            }
        };

        let peers_ref = self.peers.clone();
        
        tokio::spawn(async move{
            select! {
            _ = incoming => {},
            _ = outgoing => {}
        }

            println!("Peer {:?} disconnected.", addr);
            let mut peers = peers_ref.write();
            peers.retain(|peer| peer.addr != addr);
        });
    }
}


