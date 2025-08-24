use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, ordering::PintOrdering, validator::validtx::ValidPoolTransaction};

#[derive(Default, Debug)]
pub struct PendingPool {
    ordering: PintOrdering,
    submission_id: u64,
    independent: BTreeMap<TransactionId, PendingTransaction>,
}

impl PendingPool {

    pub fn add_transaction(
        &mut self,
        tx: Arc<ValidPoolTransaction>,
        // Base fee of blocks. If Tx fee is under this, It should rejected!
    ) {
        assert!(
            !self.contains(tx.tid()),
            "transaction already included {:?}",
            self.independent.get(tx.tid()).unwrap().transaction
        );

        let tx_id = *tx.tid();
        let submission_id = self.next_id();
        let priority = self.ordering.priority(&tx);

        let tx = PendingTransaction {
            submission_id,
            transaction: tx,
            priority,
        };

        self.independent.insert(tx_id, tx);
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction>> {
        let tx = self.independent.remove(id)?;
        Some(tx.transaction)
    }

    fn contains(&self, id: &TransactionId) -> bool {
        self.independent.contains_key(id)
    }

    const fn next_id(&mut self) -> u64 {
        let id = self.submission_id;
        self.submission_id = self.submission_id.wrapping_add(1);
        id
    }

    pub fn len(&self) -> usize {
        self.independent.len()
    }
}

#[derive(Debug)]
pub struct PendingTransaction {
    submission_id: u64,
    transaction: Arc<ValidPoolTransaction>,
    priority: u128,
}