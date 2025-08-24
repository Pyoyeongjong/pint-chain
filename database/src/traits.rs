use primitives::types::{Account, Address};

use crate::error::DatabaseError;

pub trait Database: Clone {
    fn block_number(&self) -> u64;
    fn basic(&self, address: &Address) -> Result<Option<Account>, DatabaseError>;
}