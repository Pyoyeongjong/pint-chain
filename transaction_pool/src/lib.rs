use std::sync::Arc;

use primitives::{transaction::Recovered, types::TxHash};
use provider::{Database, ProviderFactory};

use crate::{error::{PoolError, PoolErrorKind, PoolResult}, identifier::TransactionOrigin, pool::PoolInner, validator::TransactionValidationOutcome};

pub mod pool;
pub mod validator;
pub mod identifier;
pub mod ordering;
pub mod mock;
pub mod error;

#[derive(Debug, Clone)]
pub struct Pool<DB: Database> {
    pool: Arc<PoolInner<DB>>,
}

impl<DB: Database> Pool<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            pool: Arc::new(PoolInner::new(provider)),
        }
    }


    pub fn add_transaction(&self, origin: TransactionOrigin, transaction: Recovered) -> PoolResult<TxHash>{
        let (_hash, outcome) = self.validate(origin, transaction);
        match outcome {
            TransactionValidationOutcome::Valid { transaction, balance, nonce } => {
                self.pool.pool().write().add_transaction(transaction, balance, nonce)
            }
            TransactionValidationOutcome::Invalid{ transaction, error} => {
                let pool_error = PoolError {
                    hash: transaction.hash(),
                    kind: PoolErrorKind::InvalidPoolTransactionError(error),
                };
                return Err(pool_error);
            }
            TransactionValidationOutcome::UnexpectedError(tx_hash) => {
                let pool_error = PoolError {
                    hash: tx_hash,
                    kind: crate::error::PoolErrorKind::ImportError,
                };
                return Err(pool_error);
            }
        }
    }

    pub fn validate(&self, origin: TransactionOrigin, transaction: Recovered) -> (TxHash, TransactionValidationOutcome) {
        let hash = transaction.hash();
        let outcome = self.pool.validator().validate_transaction(origin, transaction);

        (hash, outcome)
    }
}


