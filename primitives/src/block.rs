
use core::hash;

use sha2::{Digest, Sha256};

use crate::error::{BlockImportError, BlockValidatioError};
use crate::transaction::Recovered;
use crate::{transaction::SignedTransaction, types::BlockHash};
use crate::types::{Address, B256};

/// Block hash
#[derive(Debug, Default)]
pub struct Header {
    pub previous_hash: BlockHash, // 32
    pub transaction_root: B256, // 32
    pub state_root: B256, // 32
    pub timestamp: u64, // 8
    pub proposer: Address, // 20
    pub nonce: u64, // 8
    pub difficulty: u32, // 4
    pub height: u64, // 8
}

impl Header {
    pub fn encode(&self) -> Vec<u8> {
        let mut raw = [0u8; 144];
        raw[0..32].copy_from_slice(&self.previous_hash.to_string().as_bytes());
        raw[32..64].copy_from_slice(&self.transaction_root.to_string().as_bytes());
        raw[64..96].copy_from_slice(&self.state_root.to_string().as_bytes());
        raw[96..104].copy_from_slice(&self.timestamp.to_be_bytes());
        raw[104..124].copy_from_slice(&self.proposer.get_addr());
        raw[124..132].copy_from_slice(&self.nonce.to_be_bytes());
        raw[132..136].copy_from_slice(&self.difficulty.to_be_bytes());
        raw[136..144].copy_from_slice(&self.height.to_be_bytes());
        raw.to_vec()
    }

    pub fn calculate_hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(self.previous_hash);
        hasher.update(self.transaction_root);
        hasher.update(self.state_root);
        hasher.update(self.timestamp.to_string().as_bytes());
        hasher.update(self.proposer.get_addr());
        hasher.update(self.nonce.to_string().as_bytes());
        hasher.update(self.difficulty.to_string().as_bytes());
        hasher.update(self.height.to_string().as_bytes());
        B256::from_slice(&hasher.finalize())
    }
}

#[derive(Debug)]
/// Block Structure 
pub struct Block{
    pub header: Header,
    pub body: Vec<Recovered>,
}

impl Block {
    pub fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        let header = self.header.encode();
        res = [res, header].concat();

        // Recoverd -> SignedTransaction
        for recovered in self.body.iter() {
            let encoded = recovered.tx().encode();
            res = [res, encoded].concat();
        }
        res
    }

    pub fn header(&self) -> &Header{
        &self.header
    }
}

/// Block hash
#[derive(Debug, Default)]
pub struct PayloadHeader {
    pub previous_hash: BlockHash, 
    pub transaction_root: B256,
    pub proposer: Address, 
    pub difficulty: u32, 
    pub height: u64, 
}

#[derive(Debug)]
/// Payload Structure (Before Mining)
pub struct Payload {
    pub header: PayloadHeader,
    pub body: Vec<SignedTransaction>,
}


/// Block Importer trait
pub trait BlockImportable: Send + Sync {
    type B;
    fn import_block(&self, block: Self::B) -> Result<BlockValidationResult, BlockImportError>;
}

pub struct BlockValidationResult {
    pub success: bool,
    pub error: Option<BlockValidatioError>,
}

impl BlockValidationResult {
    pub fn success(&mut self) {
        self.success = true;
    }

    pub fn failed(&mut self) {
        self.success = false;
    }

    pub fn add_error(&mut self, e: BlockValidatioError) {
        self.error = Some(e)
    }
}

impl Default for BlockValidationResult {
    fn default() -> Self {
        Self { success: false, error: Some(BlockValidatioError::DefaultError) }
    }
}