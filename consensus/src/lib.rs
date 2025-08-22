use std::sync::Arc;

use network::NetworkHandle;
use payload::PayloadBuilder;
use primitives::block::{Block, BlockImportable};
use provider::{Database, ProviderFactory};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use transaction_pool::Pool;

use crate::{importer::BlockImporter, miner::{MinerHandle, MinerResultMessage}};

pub mod miner;
pub mod importer;

#[derive(Debug)]
pub struct ConsensusEngine<DB: Database> {
    network: NetworkHandle,
    builder: PayloadBuilder<DB>,
    importer: BlockImporter<DB>,
    pool: Pool<DB>,
    miner_tx: MinerHandle,
    consensus_rx: UnboundedReceiver<MinerResultMessage>,
}

impl<DB: Database> ConsensusEngine<DB> {
    pub fn new(pool: Pool<DB>, builder: PayloadBuilder<DB>, network: NetworkHandle, provider: ProviderFactory<DB>, miner_tx: MinerHandle, consensus_rx: UnboundedReceiver<MinerResultMessage>) -> Self {
        Self {
            network,
            builder,
            importer: BlockImporter::new(provider),
            pool,
            miner_tx,
            consensus_rx,
        }
    }
}

impl<DB: Database > BlockImportable for ConsensusEngine<DB> where DB: Database + Send + Sync + 'static{
    type B = Block;

    fn import_block(&self, block: Self::B) -> Result<(), primitives::error::BlockImportError> {
        todo!()
    }
}

pub enum ConsensusHandleMessage {}

pub struct ConsensusHandle {
    inner: Arc<ConsensusInner>,
}

pub struct ConsensusInner {
    to_manager_tx: UnboundedSender<ConsensusHandleMessage>,
}