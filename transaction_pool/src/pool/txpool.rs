use std::{collections::HashMap, sync::Arc};

use crate::{identifier::TransactionId, pool::{parked::ParkedPool, pending::PendingPool, state::SubPool}, validator::validtx::ValidPoolTransaction};

#[derive(Debug)]
pub struct TxPool {
    all_transaction: AllTransaction,
    pending_pool: PendingPool,
    parked_pool: ParkedPool,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            all_transaction: AllTransaction::default(),
            pending_pool: PendingPool::default(),
            parked_pool: ParkedPool::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct AllTransaction {
    txs: HashMap<TransactionId, PoolInternalTransaction>,
}

impl AllTransaction {}

#[derive(Debug)]
pub struct PoolInternalTransaction {
    transaction: Arc<ValidPoolTransaction>,
    sub_pool: SubPool,
}