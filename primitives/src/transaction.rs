use crate::{signature::Signature, types::{Address, ChainId, TxHash, U256}};

/// Raw Transaction
#[derive(Debug)]
pub struct Transaction {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub to: Address,
    pub fee: u128,
    pub value: U256,
}

/// Transaction with Signature
#[derive(Debug)]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Signature,
    pub hash: TxHash,
}
