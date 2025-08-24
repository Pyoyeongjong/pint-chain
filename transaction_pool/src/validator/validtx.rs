use std::sync::Arc;

use primitives::{transaction::SignedTransaction, types::{TxHash, U256}};

use crate::identifier::{SenderId, TransactionId, TransactionOrigin};

#[derive(Debug, Clone)]
pub struct ValidPoolTransaction {
    pub transaction: SignedTransaction,
    pub transaction_id: TransactionId,
    pub origin: TransactionOrigin,
    pub timestamp: std::time::Instant,
}

impl ValidPoolTransaction {

    pub fn tx(&self) -> &SignedTransaction {
        &self.transaction
    }

    pub fn tid(&self) -> &TransactionId {
        &self.transaction_id
    }

    pub fn sender(&self) -> SenderId {
        self.tid().sender.clone()
    }

    pub fn hash(&self) -> TxHash {
        self.tx().hash
    }

    pub fn nonce(&self) -> u64 {
        self.tx().transaction().nonce
    }

    pub fn fee(&self) -> u128 {
        self.tx().transaction().fee
    }

    pub fn value(&self) -> U256 {
        self.tx().transaction().value
    }

    pub fn is_underpriced(&self, other: &Self) -> bool {
        self.fee() < other.fee()
    }
}
