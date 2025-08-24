// This project use alloy_primitives in only this file.
pub use alloy_primitives::{B256, U256};
use rand::Rng;

use crate::error::AddressError;

pub type ChainId = u64;
pub type TxHash = B256;
pub type BlockHash = B256;

const ADDR_LEN: usize = 20;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address([u8; ADDR_LEN]);

impl Address {
    pub fn from_byte(address: [u8; 20]) -> Self {
        Self(address)
    }

    pub fn from_hex(address: String) -> Result<Self, AddressError> {
        let bytes = hex::decode(address)?;
        if bytes.len() != ADDR_LEN {
            return Err(AddressError::InvalidLength(bytes.len()));
        }

        let arr: [u8; ADDR_LEN] = bytes.try_into().unwrap();
        Ok(Address(arr))
    }

    pub fn random() -> Self {
        let mut arr = [0u8; 20];
        let mut rng = rand::rng();
        rng.fill(&mut arr);
        Self(arr)
    }

    pub fn get_addr_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn get_addr(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}

impl Account {

    pub fn new(nonce: u64, balance: U256) -> Self {
        Self { nonce, balance }
    }
    pub fn update(&mut self, nonce: u64, balance: U256) {
        self.nonce = nonce;
        self.balance = balance;
    }
}