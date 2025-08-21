// This project use alloy_primitives in only this file.
pub use alloy_primitives::{B256, U256};

pub type ChainId = u64;
pub type TxHash = B256;
pub type BlockHash = B256;

const ADDR_LEN: usize = 20;
pub struct Address([u8; ADDR_LEN]);

pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}