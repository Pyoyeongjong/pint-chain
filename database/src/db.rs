use std::{collections::{BTreeMap, HashMap}, sync::RwLock};

use primitives::types::{Account, Address};

use crate::traits::Database;

pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
}

impl Database for InMemoryDB {}