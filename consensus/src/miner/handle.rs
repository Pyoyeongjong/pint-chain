use std::sync::Arc;

use primitives::handle::MinerHandleMessage;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct MinerHandle {
    inner: Arc<MinerInner>,
}

impl MinerHandle {
    pub fn new(miner_tx: UnboundedSender<MinerHandleMessage>) -> Self {
        Self {
            inner: Arc::new(MinerInner { to_manager_tx: miner_tx })
        }
    }
}

#[derive(Debug)]
pub struct MinerInner {
    to_manager_tx: UnboundedSender<MinerHandleMessage>,
}


