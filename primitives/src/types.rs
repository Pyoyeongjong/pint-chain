use std::ops::Add;

// This project use alloy_primitives in only this file.
pub use alloy_primitives::{B256, U256};
use rand::Rng;

use crate::error::AddressError;

pub type ChainId = u64;
pub type TxHash = B256;
pub type BlockHash = B256;
pub type PayloadId = u64;

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

    // This is for dev/test code
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

impl Default for Address {
    fn default() -> Self {
        let addr = [0u8; 20];
        Self::from_byte(addr)
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

    pub fn balance(&self) -> U256 {
        self.balance
    }

    pub fn nonce(&self) -> U256 {
        self.balance
    }

    pub fn sub_balance(&mut self, value: U256) {
        if value > self.balance {
            self.balance = U256::ZERO;
        } else {
            self.balance -= value;
        }
    }

    pub fn add_balance(&mut self, value: U256) {
        if self.balance > U256::MAX - value {
            self.balance = U256::MAX;
        } else {
            self.balance += value;
        }
    }

    pub fn increase_nonce(&mut self) {
        self.nonce += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Address;

    #[test]
    fn make_random_address() {
        for _ in 0..5 {
            let addr = Address::random();
            dbg!(addr.get_addr_hex());
        }
    }
}