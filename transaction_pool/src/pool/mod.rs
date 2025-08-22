use provider::{Database, ProviderFactory};

use crate::{pool::txpool::TxPool, validator::Validator};

pub mod txpool;
pub mod pending;
pub mod parked;
pub mod state;

#[derive(Debug)]
pub struct PoolInner<DB: Database> {
    validator: Validator<DB>,
    transaction_pool: TxPool,
}

impl<DB: Database> PoolInner<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            validator: Validator::new(provider),
            transaction_pool: TxPool::new(),
        }
    }
}

