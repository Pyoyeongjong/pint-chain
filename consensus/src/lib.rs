use std::time::Duration;

use payload::handle::{PayloadBuilderHandle, };
use primitives::{block::{Block, Payload}, error::BlockImportError, handle::{ConsensusHandleMessage, Handle, MinerHandleMessage, MinerResultMessage, NetworkHandleMessage, PayloadBuilderHandleMessage, PayloadBuilderResultMessage}};
use provider::{DatabaseTrait, ProviderFactory};
use tokio::sync::mpsc::UnboundedReceiver;
use transaction_pool::Pool;

use crate::{handle::ConsensusHandle, importer::BlockImporter, miner::handle::MinerHandle};

pub mod miner;
pub mod importer;
pub mod handle;

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
        builder_events: UnboundedReceiver<PayloadBuilderResultMessage>

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

    pub fn start_consensus(self, consensus_handle: ConsensusHandle, mut rx: UnboundedReceiver<ConsensusHandleMessage>) -> ConsensusHandle{

        let consensus_handle_cloned = consensus_handle.clone();

        tokio::spawn(async move {
            println!("[Consensus] Consensus channel starts.");
            let consensus_handle = consensus_handle_cloned;
            let Self { 
                importer, 
                pool, 
                network, 
                builder_handle, 
                mut builder_events, 
                miner_handle, 
                mut miner_events ,
                latest_payload
            } = self;

            // initial functions
            if latest_payload.is_none() {
                builder_handle.send(PayloadBuilderHandleMessage::BuildPayload);
            }
            let mut latest_payload: Option<Payload> = None;

            loop {
                
                tokio::select! {
                    Some(msg) = miner_events.recv() => {
                        println!("[Consensus] received message from Miner: {:?}", msg);
                        match msg {
                            MinerResultMessage::MiningSuccess(header) => {
                                println!("[Consensus] Accepted mining result!");

                                // check this result matches current payload
                                let new_latest_payload = latest_payload.clone();

                                if new_latest_payload.is_none() {
                                    continue;
                                }

                                let payload: Payload = new_latest_payload.unwrap();
                                if header.timestamp != payload.header.timestamp {
                                    eprintln!("[Consensus] Latest_payload and mining results is different.");
                                    continue;
                                }

                                let block = Block {
                                    header: header,
                                    body: payload.body,
                                };

                                consensus_handle.send(ConsensusHandleMessage::ImportBlock(block));
                            }
                        }
                    }

                    Some(msg) = builder_events.recv() => {
                        println!("[Consensus] received message from PayloadBuilder: {:?}", msg);
                        match msg {
                            PayloadBuilderResultMessage::Payload(payload) => {
                                println!("[Consensus] Accepted payload");
                                if payload.body.len() == 0 {
                                    println!("[Consensus] Payload with no body. Wait for new Transaction..");
                                    let builder_handle_cloned = builder_handle.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(Duration::from_secs(5)).await;
                                        builder_handle_cloned.send(PayloadBuilderHandleMessage::BuildPayload);
                                    });
                                    continue;
                                }
                                let miner_handle_cloned = miner_handle.clone();
                                miner_handle_cloned.send(MinerHandleMessage::NewPayload(payload.header.clone()));
                                latest_payload = Some(payload);
                            }
                            PayloadBuilderResultMessage::PoolIsEmpty => {
                                let builder_handle_cloned = builder_handle.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                    builder_handle_cloned.send(PayloadBuilderHandleMessage::BuildPayload);
                                });
                            }
                        }
                    }

                    Some(msg) = rx.recv() => {
                        println!("[Consensus] received message: {:?}", msg);
                        match msg {
                            ConsensusHandleMessage::ImportBlock(block) => {
                                // if this is a not succeeding block, ask for network to get new blockchain data
                                if let Err(e) = importer.import_new_block(block.clone()) {
                                    match e {
                                        BlockImportError::BlockHeightError => {
                                            eprintln!("[Consensus] Failed to import new block due to block heignt: {:?}. Try to update new datas.", e);
                                            network.send(NetworkHandleMessage::RequestData);
                                            continue;
                                        }
                                        BlockImportError::AlreadyImportedBlock => {
                                            println!("[Consensus] Already imported block! {:?}", block.header.height);
                                            continue;
                                        }
                                        _ => {
                                            eprintln!("[Consensus] Failed to import new block: {:?}", e);
                                        }
                                    }
                                }
                                pool.remove_block_transactions(&block);
                                pool.reorganize_pool();
                                latest_payload = None;
                                network.send(NetworkHandleMessage::BroadcastBlock(block));
                                builder_handle.send(PayloadBuilderHandleMessage::BuildPayload);
                            }
                            ConsensusHandleMessage::NewTransaction(_recovered) => {
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