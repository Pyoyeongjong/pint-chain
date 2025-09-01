use parking_lot::RwLock;
use provider::{Database, ProviderFactory};

use crate::{pool::{best::BestTransactions, txpool::TxPool}, validator::Validator};

pub mod txpool;
pub mod pending;
pub mod parked;
pub mod state;
pub mod best;

#[derive(Debug)]
pub struct PoolInner<DB: Database> {
    validator: Validator<DB>,
    transaction_pool: RwLock<TxPool>,
}

impl<DB: Database> PoolInner<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            validator: Validator::new(provider),
            transaction_pool: RwLock::new(TxPool::new()),
        }
    }

    pub fn validator(&self) -> &Validator<DB> {
        &self.validator
    }

    pub fn pool(&self) -> &RwLock<TxPool> {
        &self.transaction_pool
    }

    pub fn best_transactions(&self) -> BestTransactions {
        self.pool().read().best_transactions()
    }
}

