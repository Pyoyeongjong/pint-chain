use std::{collections::HashMap};

use primitives::{block::{Block, Header}, types::{Account, Address}, world::World};

use crate::error::DatabaseError;

pub trait Database: Send + Sync + Clone + 'static {
    fn latest_block_number(&self) -> u64;
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>>;
    fn get_state(&self, block_no: u64) -> Result<(Option<HashMap<Address, Account>>, Option<World>), crate::error::DatabaseError>;
    fn get_block(&self, block_no: u64) -> Result<Block, DatabaseError>;
    fn update(&self, new_account_state: HashMap<Address, Account>, new_field_state: World, new_block: Block);
    fn get_latest_block_header(&self) -> Header;
}