use std::sync::Arc;

use primitives::block::{Block, BlockImportable};
use provider::Database;
use tokio::{net::TcpListener, sync::mpsc::UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use transaction_pool::Pool;

use crate::peer::Peer;

pub mod peer;

pub struct NetworkManager<DB: Database> {
    listener: TcpListener,
    handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    peers: Vec<Peer>,
    consensus: Box<dyn BlockImportable<B = Block>>,
}

pub struct NetworkHandle {
    inner: Arc<NetworkInner>,
}

pub struct NetworkInner {
    to_manager_tx: UnboundedSender<NetworkHandleMessage>,
}

pub enum NetworkHandleMessage {}
