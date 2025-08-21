use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

pub struct MinerHandle {
    inner: Arc<MinerInner>,
}

pub struct MinerInner {
    to_manager_tx: UnboundedSender<MinerHandleMessage>,
}

pub enum MinerHandleMessage {}

pub enum MinerResultMessage {}