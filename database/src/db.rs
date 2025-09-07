use std::{collections::{BTreeMap, HashMap}, sync::Arc};

use parking_lot::{RwLock};
use primitives::{block::Block, types::{Account, Address, U256}, world::World};

use crate::{error::DatabaseError, traits::Database};

#[derive(Debug)]
pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
    field: RwLock<BTreeMap<u64, World>>,
    blockchain: RwLock<BTreeMap<u64, Block>>,
    latest: RwLock<u64>,
}

impl InMemoryDB {

    pub fn encode_block(&self, block_no: u64) -> Result<Vec<u8>, DatabaseError> {
        if block_no > *self.latest.read() {
            return Err(DatabaseError::BlockEncodeError);
        }
        let binding= self.accounts.read();
        let accounts = binding.get(&block_no).unwrap();
        
        let binding = self.field.read();
        let field = binding.get(&block_no).unwrap();

        let mut res = Vec::new();
        for (address, account) in accounts.iter() {
            let addr = address.get_addr();
            let account = account.encode();
            res.extend_from_slice(addr);
            res.extend_from_slice(&account);
        };

        res.extend_from_slice(&field.encode());

        Ok(res)
    }

    // 28dcb1338b900419cd613a8fb273ae36e7ec2b1d pint
    // 0534501c34f5a0f3fa43dc5d78e619be7edfa21a chain
    // 08041f667c366ee714d6cbefe2a8477ad7488f10 apple
    // b2aaaf07a29937c3b833dca1c9659d98a9569070 banana
    pub fn genesis_state() -> Self {
        let mut db = Self::new();
        let account = Account {
            nonce: 0,
            balance: U256::from(1000000000),
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

        let mut blockchain: BTreeMap<u64, Block> = BTreeMap::new();
        let genesis_block = Block::genesis_block();
        blockchain.insert(0, genesis_block);

        Self {
            accounts: RwLock::new(accounts), 
            field: RwLock::new(field), 
            blockchain: RwLock::new(blockchain),
            latest: RwLock::new(0),
        }
    }

    pub fn add_account(&mut self, address: Address, account: Account) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts = state.entry(*self.latest.read()).or_default();
        latest_accounts.insert(address, account);
        
        Ok(())
    }
}

impl Database for Arc<InMemoryDB> {
    fn latest_block_number(&self) -> u64 {
        *self.latest.read()
    }
    
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts: &mut HashMap<Address, Account> = state.entry(self.latest_block_number()).or_default();
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

    fn get_block(&self, block_no: u64) -> Result<Block, DatabaseError> {
        let blockchain = self.blockchain.read();
        if let Some(block) = blockchain.get(&block_no) {
            Ok(block.clone())
        } else {
            Err(DatabaseError::DataNotExists)
        }
    }

    fn get_header(&self, block_no: u64) -> Result<primitives::block::Header, DatabaseError> {
        let blockchain = self.blockchain.read();
        if let Some(block) = blockchain.get(&block_no) {
            Ok(block.header().clone())
        } else {
            Err(DatabaseError::DataNotExists)
        }
    }

    fn get_latest_block_header(&self) -> primitives::block::Header {
        let blockchain = self.blockchain.read();
        let latest = self.latest_block_number();
        let block = blockchain.get(&latest).unwrap();
        block.header.clone()
    }
    
    fn update(&self, new_account_state: HashMap<Address, Account>, new_field_state: World, block: Block) {
        let mut latest = self.latest.write();
        *latest += 1;
        let mut state = self.accounts.write();
        state.insert(*latest, new_account_state);

        let mut field = self.field.write();
        field.insert(*latest, new_field_state);

        let mut blockchain = self.blockchain.write();
        blockchain.insert(*latest, block);
        println!("DB updated {}", latest);
    }
}