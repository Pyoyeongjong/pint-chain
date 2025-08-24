use std::time::Instant;

use primitives::transaction::{SignedTransaction, Transaction};

use crate::{identifier::{TransactionId, TransactionOrigin}, validator::validtx::ValidPoolTransaction};

#[derive(Default)]
pub struct MockValidator;

impl MockValidator {

    pub fn validate(&mut self, tx: SignedTransaction) -> ValidPoolTransaction {
        let tid = TransactionId {
            sender: tx.recover_signer().unwrap(),
            nonce: tx.transaction().nonce,
        };
        ValidPoolTransaction {
            transaction: tx,
            transaction_id: tid,
            origin: TransactionOrigin::External,
            timestamp: Instant::now(),
        }
    }
}