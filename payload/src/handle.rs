use std::sync::Arc;

use primitives::handle::PayloadBuilderHandleMessage;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct PayloadBuilderHandle {
    inner: Arc<PayloadBuilderInner>,
}

impl PayloadBuilderHandle {
    pub fn new(to_manager_tx: UnboundedSender<PayloadBuilderHandleMessage>) -> Self {
        Self {
            inner: Arc::new(PayloadBuilderInner {
                to_manager_tx,
            })
        }
    }
}

#[derive(Debug)]
pub struct PayloadBuilderInner {
    to_manager_tx: UnboundedSender<PayloadBuilderHandleMessage>,
}
