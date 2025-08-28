use payload::handle::{PayloadBuilderHandle, };
use primitives::{block::{Block, BlockImportable, BlockValidationResult}, error::BlockImportError, handle::{ConsensusHandleMessage, Handle, MinerResultMessage, NetworkHandleMessage, PayloadBuilderResultMessage}};
use provider::{Database, ProviderFactory};
use tokio::sync::mpsc::{self, UnboundedReceiver };
use transaction_pool::Pool;

use crate::{handle::ConsensusHandle, importer::BlockImporter, miner::handle::MinerHandle};

pub mod miner;
pub mod importer;
pub mod handle;

#[derive(Debug)]
pub struct ConsensusEngine<DB: Database> {
    importer: BlockImporter<DB>,
    pool: Pool<DB>,
    // Network
    network: Box<dyn Handle<Msg = NetworkHandleMessage>>,
    // PayloadBuilder
    builder_handle: PayloadBuilderHandle,
    builder_events: UnboundedReceiver<PayloadBuilderResultMessage>,
    // Miner
    miner_handle: MinerHandle,
    miner_events: UnboundedReceiver<MinerResultMessage>,
}

impl<DB: Database> ConsensusEngine<DB> {
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
            miner_handle,
            miner_events,
            builder_events,
        }
    }

    pub fn start_consensus(self) -> ConsensusHandle{

        let (tx, mut rx) = mpsc::unbounded_channel::<ConsensusHandleMessage>();

        let consensus_handle = ConsensusHandle::new(tx);

        tokio::spawn(async move {
            println!("Consensus channel starts.");
            let Self { 
                importer, 
                pool, 
                network, 
                builder_handle, 
                mut builder_events, 
                miner_handle, 
                mut miner_events 
            } = self;

            loop {
                tokio::select! {
                    Some(msg) = miner_events.recv() => {

                    }

                    Some(msg) = builder_events.recv() => {

                    }

                    Some(msg) = rx.recv() => {
                        match msg {
                            ConsensusHandleMessage::ImportBlock(block) => {
                                let res = importer.validate_block(block);
                            }
                            ConsensusHandleMessage::NewTransaction(recovered) => {
                                // revise current payload (maybe?)
                            }
                        }
                    }
                }
            }
        });

        consensus_handle
    }
}

impl<DB: Database> BlockImportable for ConsensusEngine<DB> 
{
    type B = Block;

    fn import_block(&self, block: Self::B) -> Result<BlockValidationResult, BlockImportError> {
        self.importer.validate_block(block)
    }
}
