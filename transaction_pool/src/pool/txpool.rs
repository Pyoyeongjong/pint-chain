use std::{collections::HashMap, sync::Arc};

use crate::{identifier::TransactionId, pool::{parked::ParkedPool, pending::PendingPool, state::SubPool}, validator::validtx::ValidPoolTransaction};

pub struct TxPool {
    all_transaction: AllTransaction,
    pending_pool: PendingPool,
    parked_pool: ParkedPool,
}

pub struct AllTransaction {
    txs: HashMap<TransactionId, PoolInternalTransaction>,
}

pub struct PoolInternalTransaction {
    transaction: Arc<ValidPoolTransaction>,
    sub_pool: SubPool,
}