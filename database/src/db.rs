use std::{collections::{BTreeMap, HashMap}, ops::Add, sync::Arc};

use parking_lot::RwLock;
use primitives::{types::{Account, Address, U256}, world::World};

use crate::{traits::Database};

#[derive(Debug)]
pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
    field: RwLock<BTreeMap<u64, World>>,
    latest: u64,
}

impl InMemoryDB {

    // 28dcb1338b900419cd613a8fb273ae36e7ec2b1d pint
    // 0534501c34f5a0f3fa43dc5d78e619be7edfa21a chain
    // 08041f667c366ee714d6cbefe2a8477ad7488f10 apple
    // b2aaaf07a29937c3b833dca1c9659d98a9569070 banana
    pub fn genesis_block() -> Self {
        let mut db = Self::new();
        let account = Account {
            nonce: 0,
            balance: U256::from(100000000),
        };
        let address = Address::from_hex("28dcb1338b900419cd613a8fb273ae36e7ec2b1d".to_string()).unwrap();
        db.add_account(address, account.clone()).unwrap();
        let address = Address::from_hex("0534501c34f5a0f3fa43dc5d78e619be7edfa21a".to_string()).unwrap();
        db.add_account(address, account.clone()).unwrap();
        db
    }


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

    pub fn add_account(&mut self, address: Address, account: Account) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts = state.entry(self.latest).or_default();
        latest_accounts.insert(address, account);
        
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
    
    fn get_state(&self, block_no: u64) -> Result<(Option<HashMap<Address, Account>>, Option<World>), crate::error::DatabaseError> {
        let accounts = self.accounts.read();
        let mut account_base = None;
        if let Some(state_account) = accounts.get(&block_no) {
            account_base = Some(state_account.clone());
        }

        let field = self.field.read();
        let mut field_base = None;
        if let Some(state_field) = field.get(&block_no) {
            field_base = Some(state_field.clone());
        }

        Ok((account_base, field_base))
    }
}