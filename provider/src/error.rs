use database::error::DatabaseError;
use primitives::types::TxHash;

#[derive(Clone)]
pub enum ExecutionError {
    StateExecutionError(StateExecutionError)
}

pub enum ProviderError {
    DatabaseError(DatabaseError),
    StateNotExist(u64)
}

#[derive(Clone)]
pub enum StateExecutionError {
    TransactionExecutionError(TxHash, TxExecutionError)
}

#[derive(Clone)]
pub enum TxExecutionError {
    SenderHasNotEnoughBalance,
    SenderHasNoAccount,
    NonceError,
}