use std::{net::SocketAddr, sync::{Arc}};

use parking_lot::RwLock;
use primitives::handle::Handle;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::{mpsc::{self, UnboundedSender}}};

use crate::{NetworkHandle, NetworkHandleMessage};

#[derive(Debug, Clone)]
pub struct Peer {
    id: u64,
    addr: SocketAddr,
    tx: UnboundedSender<NetworkHandleMessage>,
}

impl Peer {
    pub fn new(id: u64, addr: SocketAddr, tx: UnboundedSender<NetworkHandleMessage>) -> Self {
        Self { id ,addr, tx }
    }

    pub fn send(&self, msg: NetworkHandleMessage) {
        println!("#Network# send {:?} with {:?}", self.addr, msg);
        if let Err(e) = self.tx.send(msg) {
            eprintln!("Failed to send NetworkHandleMessage: {:?}", e);
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn update_addr(&mut self, addr: SocketAddr) {
        self.addr = addr;
    }
}


#[derive(Debug)]
pub struct PeerList {
    pub submission_id: u64,
    pub peers: Arc<RwLock<Vec<Peer>>>,
}

impl PeerList {
    pub fn new() -> Self {
        Self {
            submission_id: 0,
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

    pub fn remove_peer_by_id(&mut self, pid: u64) {
        let mut peers = self.inner().write();
        peers.retain(|peer| peer.id != pid);
    }

    pub fn get_id(&mut self) -> u64 {
        let sub_id = self.submission_id;
        self.submission_id += 1;
        sub_id
    }
}

impl PeerList {
    pub fn insert_new_peer(&mut self, socket: TcpStream, addr: SocketAddr, network_handle: NetworkHandle) -> (Peer, u64) {
        let pid = self.get_id();
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkHandleMessage>();
        // tx is used for every componets who want to send peer msg
        // rx isolates socket
        let mut peers = self.peers.write();
        let new_peer = Peer::new(pid as u64, addr.clone(), tx);
        peers.push(new_peer.clone());

        let (mut read_socket, mut write_socket) = socket.into_split();

        // incoming loop
        let incoming = async move {
            println!("Peer {:?} incoming task has spawned.", addr);
            let mut buf = [0u8; 1024];
            loop {
                match read_socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Peer {:?} closed connection", addr);
                        network_handle.send(NetworkHandleMessage::RemovePeer(pid));
                        break;
                    }
                    Ok(n) => {
                        println!("encoded {} data incomed", n);

                        match NetworkHandleMessage::decode(&buf[..n], addr) {
                            Ok(res) => {
                                match res {
                                    Some(decoded) => {
                                        let _ = network_handle.send(decoded);
                                    }
                                    None =>  {
                                        eprintln!("Invalid Request from Peer: {:?}", addr);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to decode Network handle message: {:?} from {:?}", e, addr);
                                continue;
                            }
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

        (new_peer, pid as u64)
    }
}


