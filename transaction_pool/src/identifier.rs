use primitives::types::Address;

#[derive(Debug)]
pub struct TransactionId {
    sender: Address,
    nonce: u64,
}

#[derive(Debug)]
pub enum TransactionOrigin {
    Local,
    External,
}