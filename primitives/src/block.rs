
use crate::error::BlockImportError;
use crate::transaction::Recovered;
use crate::{transaction::SignedTransaction, types::BlockHash};
use crate::types::{Address, B256};

/// Block hash
#[derive(Debug)]
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

#[derive(Debug)]
/// Payload Structure (Before Mining)
pub struct Payload {
    pub body: Vec<SignedTransaction>,
}


/// Block Importer trait
pub trait BlockImportable: Send + Sync {
    type B;
    fn import_block(&self, block: Self::B) -> Result<BlockValidationResult, BlockImportError>;
}

pub struct BlockValidationResult {

}