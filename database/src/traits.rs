use primitives::types::{Account, Address};
pub trait Database: Clone {
    fn block_number(&self) -> u64;
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>>;
}