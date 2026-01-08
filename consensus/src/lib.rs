use payload::handle::PayloadBuilderHandle;
use primitives::{
    block::{Block, Payload},
    error::BlockImportError,
    handle::{
        ConsensusHandleMessage, Handle, MinerHandleMessage, MinerResultMessage,
        NetworkHandleMessage, PayloadBuilderHandleMessage, PayloadBuilderResultMessage,
    },
};
use provider::{DatabaseTrait, ProviderFactory};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info};
use transaction_pool::Pool;

use crate::{handle::ConsensusHandle, importer::BlockImporter, miner::handle::MinerHandle};

pub mod handle;
pub mod importer;
pub mod miner;

#[derive(Debug)]
pub struct ConsensusEngine<DB: DatabaseTrait> {
    importer: BlockImporter<DB>,
    pool: Pool<DB>,
    // Network
    network: Box<dyn Handle<Msg = NetworkHandleMessage>>,
    // PayloadBuilder
    builder_handle: PayloadBuilderHandle,
    builder_events: UnboundedReceiver<PayloadBuilderResultMessage>,
    latest_payload: Option<Payload>,
    // Miner
    miner_handle: MinerHandle,
    miner_events: UnboundedReceiver<MinerResultMessage>,
}

impl<DB: DatabaseTrait> ConsensusEngine<DB> {
    pub fn new(
        pool: Pool<DB>,
        builder_handle: PayloadBuilderHandle,
        network: Box<dyn Handle<Msg = NetworkHandleMessage>>,
        provider: ProviderFactory<DB>,
        miner_handle: MinerHandle,
        miner_events: UnboundedReceiver<MinerResultMessage>,
        builder_events: UnboundedReceiver<PayloadBuilderResultMessage>,
    ) -> Self {
        Self {
            network,
            importer: BlockImporter::new(provider),
            pool,
            builder_handle,
            latest_payload: None,
            miner_handle,
            miner_events,
            builder_events,
        }
    }

    pub fn start_consensus(
        self,
        consensus_handle: ConsensusHandle,
        mut rx: UnboundedReceiver<ConsensusHandleMessage>,
    ) -> ConsensusHandle {
        let consensus_handle_cloned = consensus_handle.clone();

        tokio::spawn(async move {
            info!("Consensus channel starts.");
            let consensus_handle = consensus_handle_cloned;
            let Self {
                importer,
                pool,
                network,
                builder_handle,
                mut builder_events,
                miner_handle,
                mut miner_events,
                latest_payload,
            } = self;

            // initial functions
            if latest_payload.is_none() {
                builder_handle.send(PayloadBuilderHandleMessage::BuildPayload);
            }
            let mut latest_payload: Option<Payload> = None;

            loop {
                tokio::select! {
                    Some(msg) = miner_events.recv() => {
                        info!(
                            "Received message from Miner: {}", msg
                        );
                        match msg {
                            MinerResultMessage::MiningSuccess(header) => {
                                info!{
                                    "Accepted mining result. Header: {:?}", header
                                };

                                // check this result matches current payload
                                let new_latest_payload = latest_payload.clone();

                                if new_latest_payload.is_none() {
                                    continue;
                                }

                                let payload: Payload = new_latest_payload.unwrap();
                                if header.timestamp != payload.header.timestamp {
                                    error!(
                                        "Latest_payload and mining results is different."
                                    );
                                    continue;
                                }

                                let block = Block {
                                    header: header,
                                    body: payload.body,
                                };

                                consensus_handle.send(ConsensusHandleMessage::ImportBlock(block));
                            }
                            MinerResultMessage::MiningHalted => {
                                info!(
                                    "Mining task halted"
                                );
                                builder_handle.send(PayloadBuilderHandleMessage::BuildPayload);
                            }
                        }
                    }

                    Some(msg) = builder_events.recv() => {
                        info!(
                            "Recived message from PayloadBuilder: {}", msg
                        );
                        match msg {
                            PayloadBuilderResultMessage::Payload(payload) => {
                                info!(
                                    "Accepted payload"
                                );
                                if payload.body.len() == 0 {
                                    info!(
                                        "Payload with no body. Wait for new Transaction.."
                                    );
                                } else {
                                    let miner_handle_cloned = miner_handle.clone();
                                    miner_handle_cloned.send(MinerHandleMessage::NewPayload(payload.header.clone()));
                                    latest_payload = Some(payload);
                                }

                            }
                            PayloadBuilderResultMessage::PoolIsEmpty => {
                                info!(
                                    "There are no txs in pending pool. Wait for new Transaction.."
                                );
                            }
                        }
                    }

                    Some(msg) = rx.recv() => {
                        info!(
                            "Received message: {}", msg
                        );
                        match msg {
                            ConsensusHandleMessage::ImportBlock(block) => {
                                // if this is a not succeeding block, ask for network to get new blockchain data
                                if let Err(e) = importer.import_new_block(block.clone()) {
                                    match e {
                                        BlockImportError::BlockHeightError => {
                                            error!(
                                                error = ?e,
                                                "Failed to import new block due to block height. Try to update new datas."
                                            );
                                            // TODO: 이거 왜있지?
                                            // network.send(NetworkHandleMessage::RequestData);
                                            continue;
                                        }
                                        BlockImportError::NotChainedBlock => {
                                            error!(
                                                error = ?e,
                                                "Failed to import new block due to block hash. Try to update new datas."
                                            );
                                            // TODO: 이거 왜있지?
                                            // network.send(NetworkHandleMessage::RequestData);
                                            continue;
                                        }
                                        BlockImportError::AlreadyImportedBlock => {
                                            info!(
                                                height = &block.header.height,
                                                "Already imported block!"
                                            );
                                            continue;
                                        }
                                        _ => {
                                            error!(
                                                error = ?e,
                                                "Failed to import new block."
                                            );
                                        }
                                    }
                                }
                                pool.remove_block_transactions(&block);
                                pool.reorganize_pool();
                                latest_payload = None;
                                miner_handle.send(MinerHandleMessage::HaltMining);
                                network.send(NetworkHandleMessage::BroadcastBlock(block));
                            }
                            ConsensusHandleMessage::NewTransaction(_recovered) => {
                                // 무조건 보낼까?
                                builder_handle.send(PayloadBuilderHandleMessage::BuildPayload);
                                // update current payload (maybe?)
                                // pass now
                            }
                        }
                    }
                }
            }
        });

        consensus_handle
    }
}
