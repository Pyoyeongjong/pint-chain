use primitives::types::Address;

pub struct TransactionId {
    sender: Address,
    nonce: u64,
}

pub enum TransactionOrigin {
    Local,
    External,
}