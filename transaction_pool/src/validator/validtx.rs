use primitives::transaction::SignedTransaction;

use crate::identifier::{TransactionId, TransactionOrigin};

pub struct ValidPoolTransaction {
    transaction: SignedTransaction,
    transaction_id: TransactionId,
    origin: TransactionOrigin,
    timestamp: std::time::Instant,
}