pub mod handle;

use std::sync::{atomic::AtomicU64, Arc};

use primitives::{handle::{MinerHandleMessage, MinerResultMessage}, types::{B256, U256}};
use sha2::{digest::consts::U25, Digest, Sha256};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::miner::handle::MinerHandle;

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
            println!("Miner chainnel starts.");
            let Miner { mut miner_rx, consensus_tx, epoch, worker } = self;
            loop {
                if let Some(msg) = miner_rx.recv().await {
                    println!("Miner received message: {:?}", msg);
                    match msg {
                        MinerHandleMessage::NewPayload(payload_header) => {
                            // spawn payload mining task
                            let consensus_tx = consensus_tx.clone();
                            let _epoch = epoch.clone();
                            let _worker = worker.clone();

                            // this order should be same as header hash function
                            let mut hasher = Sha256::new();
                            hasher.update(payload_header.previous_hash);
                            hasher.update(payload_header.transaction_root);
                            hasher.update(payload_header.state_root);
                            hasher.update(payload_header.timestamp.to_be_bytes());
                            hasher.update(payload_header.proposer.get_addr());
                            hasher.update(payload_header.difficulty.to_be_bytes());
                            hasher.update(payload_header.height.to_be_bytes());

                            tokio::spawn(async move {
                                let mut nonce: u64 = 0;
                                let difficulty = payload_header.difficulty;
                                loop {
                                    let mut new_hasher = hasher.clone();
                                    new_hasher.update(nonce.to_be_bytes());
                                    let result = B256::from_slice(&new_hasher.finalize());
                                    if meets_target(result, difficulty) {
                                        // Mining Ok!
                                        let header = payload_header.clone().into_header(nonce);
                                        if let Err(e) = consensus_tx.send(MinerResultMessage::MiningSuccess(header)) {
                                            eprintln!("Failed to send MinerResultMessage: {:?}", e);
                                        }
                                        return;
                                    }
                                    nonce += 1;
                                }
                                
                            });
                        }
                    }
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

fn meets_target(result: B256, difficulty: u32) -> bool {
    let mut remains = difficulty;

    for byte in result.0 {
        if remains >= 8 {
            if byte != 0 {
                return false;
            } else {
                remains -= 8;
            }
        } else if remains > 0 {
            let mask = 0xFF << (8 - remains);
            if byte & mask != 0 {
                return false;
            } else {
                return true;
            }
        } else {
            return true;
        }
    }

    remains == 0
}

