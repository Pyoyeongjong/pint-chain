use std::sync::Arc;

use network::NetworkHandle;
use payload::PayloadBuilder;
use primitives::block::{Block, BlockImportable};
use provider::Database;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use transaction_pool::Pool;

use crate::{importer::BlockImporter, miner::{MinerHandle, MinerResultMessage}};

pub mod miner;
pub mod importer;

pub struct ConsensusEngine<DB: Database> {
    network: NetworkHandle,
    builder: PayloadBuilder<DB>,
    importer: BlockImporter<DB>,
    pool: Pool<DB>,
    miner_tx: MinerHandle,
    miner_rx: UnboundedReceiver<MinerResultMessage>,
    handle_rx: UnboundedReceiver<ConsensusHandleMessage>,
}

impl<DB: Database> BlockImportable for ConsensusEngine<DB> {
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