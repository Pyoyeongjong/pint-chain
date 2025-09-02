use std::{collections::HashMap, hash::Hash, ops::Add};

use primitives::{types::{Account, Address}, world::World};

pub trait Database: Send + Sync + Clone + 'static {
    fn block_number(&self) -> u64;
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>>;
    fn get_state(&self, block_no: u64) -> Result<(Option<HashMap<Address, Account>>, Option<World>), crate::error::DatabaseError>;
    fn update(&self, new_account_state: HashMap<Address, Account>, new_field_state: World);
}