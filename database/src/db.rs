use std::{collections::{BTreeMap, HashMap}, sync::{Arc}};

use parking_lot::RwLock;
use primitives::{types::{Account, Address}, world::World};

use crate::{traits::Database};

#[derive(Debug)]
pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
    field: RwLock<BTreeMap<u64, World>>,
    latest: u64,
}

impl InMemoryDB {

    pub fn new() -> Self {
        let mut accounts: BTreeMap<u64, HashMap<Address, Account>> = BTreeMap::new();
        accounts.insert(0 as u64, HashMap::new());
        
        let mut field: BTreeMap<u64, World> = BTreeMap::new();
        field.insert(0, World::new());

        Self {
            accounts: RwLock::new(accounts), 
            field: RwLock::new(field), 
            latest: 0,
        }
    }

    pub fn add_account(&mut self, address: Address, accout: Account) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts = state.entry(self.latest).or_default();
        latest_accounts.insert(address, accout);
        
        Ok(())
    }
}

impl Database for Arc<InMemoryDB> {
    fn block_number(&self) -> u64 {
        (**self).latest
    }
    
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts = state.entry(self.latest).or_default();
        Ok(latest_accounts.get(address).or(None).cloned())
    }
}