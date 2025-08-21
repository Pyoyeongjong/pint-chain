use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, ordering::PintOrdering, validator::validtx::ValidPoolTransaction};

pub struct PendingPool {
    ordering: PintOrdering,
    submission_id: u64,
    independent: BTreeMap<TransactionId, PendingTransaction>,
}

pub struct PendingTransaction {
    submission_id: u64,
    transaction: Arc<ValidPoolTransaction>,
    priority: u128,
}