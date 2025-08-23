use crate::error::BlockImportError;
use crate::{transaction::SignedTransaction, types::BlockHash};
use crate::types::{Address, B256};

/// Block hash
pub struct Header {
    pub previous_hash: BlockHash,
    pub transaction_root: B256,
    pub state_root: B256,
    pub timestamp: u64,
    pub proposer: Address,
    pub nonce: u64,
    pub difficulty: u32,
    pub height: u64,
}

/// Block Structure 
pub struct Block {
    pub header: Header,
    pub body: Vec<SignedTransaction>,
}

#[derive(Debug)]
/// Payload Structure (Before Mining)
pub struct Payload {
    pub body: Vec<SignedTransaction>,
}


/// Block Importer trait
pub trait BlockImportable: Send + Sync {
    type B;
    fn import_block(&self, block: Self::B) -> Result<(), BlockImportError>;
}