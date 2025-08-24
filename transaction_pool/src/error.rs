use std::sync::Arc;

use primitives::types::TxHash;

use crate::validator::validtx::ValidPoolTransaction;

pub type PoolResult<T> = Result<T, PoolError>;

#[derive(Debug)]
pub struct PoolError {
    pub hash: TxHash,
    pub kind: PoolErrorKind,
}

impl PoolError {
    pub fn new(hash: TxHash, kind: PoolErrorKind) -> Self {
        Self { hash, kind }
    }
}

#[derive(Debug)]
pub enum PoolErrorKind {
    AlreadyImported,
    InvalidTransaction(Arc<ValidPoolTransaction>),
    RelpacementUnderpriced(Arc<ValidPoolTransaction>),
    ImportError,
}

#[derive(Debug)]
pub enum InsertErr {
    Underpriced {
        transaction: Arc<ValidPoolTransaction>,
    },
    InvalidTransaction {
        transaction: Arc<ValidPoolTransaction>,
    },
}

#[derive(Debug)]
pub enum InvalidPoolTransactionError {
    NotEnoughFeeError,
    NonceIsNotConsistent,
}
