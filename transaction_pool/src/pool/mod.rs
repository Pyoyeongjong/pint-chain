use provider::Database;

use crate::{pool::txpool::TxPool, validator::Validator};

pub mod txpool;
pub mod pending;
pub mod parked;
pub mod state;

pub struct PoolInner<DB: Database> {
    validator: Validator<DB>,
    transaction_pool: TxPool,
}

