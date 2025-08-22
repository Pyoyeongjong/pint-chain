use std::{collections::{BTreeMap, HashMap}, sync::{Arc, RwLock}};

use primitives::types::{Account, Address};

use crate::traits::Database;

#[derive(Debug)]
pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
}

impl InMemoryDB {
    pub fn new() -> Self {
        Self {
            accounts: Default::default()
        }
    }
}

impl Database for InMemoryDB {}

impl Database for Arc<InMemoryDB> {}