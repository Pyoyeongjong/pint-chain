use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, validator::validtx::ValidPoolTransaction};

pub struct ParkedPool {
    submission_id: u64,
    by_id: BTreeMap<TransactionId, ParkedTransaction>,
}

pub struct ParkedTransaction {
    submission_id: u64,
    transaction: Arc<ValidPoolTransaction>,
}