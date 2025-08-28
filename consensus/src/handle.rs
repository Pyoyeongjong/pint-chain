use std::sync::Arc;

use primitives::handle::{ConsensusHandleMessage, Handle};
use tokio::sync::mpsc::UnboundedSender;


#[derive(Debug)]
pub struct ConsensusHandle {
    inner: Arc<ConsensusInner>,
}

impl ConsensusHandle {
    pub fn new(tx: UnboundedSender<ConsensusHandleMessage>) -> Self {
        Self {
            inner: Arc::new(ConsensusInner { to_manager_tx: tx })
        }
    }
}

impl Handle for ConsensusHandle {
    type Msg = ConsensusHandleMessage;

    fn send(&self, msg: Self::Msg) {
        if let Err(e) = self.inner.to_manager_tx.send(msg) {
            eprintln!("Failed to send ConsensusHandleMessage: {:?}", e);
        }
    }
}

#[derive(Debug)]
pub struct ConsensusInner {
    to_manager_tx: UnboundedSender<ConsensusHandleMessage>,
}