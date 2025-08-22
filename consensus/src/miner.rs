use std::sync::{atomic::AtomicU64, Arc};

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct Miner {
    miner_rx: UnboundedReceiver<MinerHandleMessage>,
    consensus_tx: UnboundedSender<MinerResultMessage>,
    epoch: Arc<AtomicU64>,
    worker: usize,
}

impl Miner {

    pub fn new(miner_rx: UnboundedReceiver<MinerHandleMessage>, consensus_tx:UnboundedSender<MinerResultMessage>) -> Self {
        Self {
            miner_rx,
            consensus_tx,
            epoch: Default::default(),
            worker: 0,
        }
    }

    pub fn start_channel(self) {
        tokio::spawn(async move{
            let Miner { mut miner_rx, consensus_tx, epoch, worker } = self;
            loop {
                if let Some(msg) = miner_rx.recv().await {
                    // TODO:
                }
            }
        });
    }

    pub fn build_miner() -> (MinerHandle, UnboundedReceiver<MinerResultMessage>) {
        let (miner_tx, miner_rx) = mpsc::unbounded_channel::<MinerHandleMessage>();
        let (consensus_tx, consensus_rx) = mpsc::unbounded_channel::<MinerResultMessage>();

        let miner_handle = MinerHandle::new(miner_tx);
        let miner = Miner::new(miner_rx, consensus_tx);

        miner.start_channel();
        (miner_handle, consensus_rx)
    }

}

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


pub enum MinerHandleMessage {}

pub enum MinerResultMessage {}