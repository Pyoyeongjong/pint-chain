use std::{collections::HashMap, sync::Arc};

use primitives::{types::{Account, Address}, world::World};


#[derive(Debug)]
pub struct State {
    accounts: Arc<HashMap<Address, Account>>,
    field: Arc<World>,
}

pub struct ExecutableState {
    accounts_base: Arc<HashMap<Address, Account>>,
    accounts_write: HashMap<Address, Account>,
    field_base: Arc<World>,
    field_writes: World,
}