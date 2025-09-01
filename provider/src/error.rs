use database::error::DatabaseError;
use primitives::types::TxHash;

#[derive(Clone, Debug)]
pub enum ExecutionError {
    StateExecutionError(StateExecutionError)
}

pub enum ProviderError {
    DatabaseError(DatabaseError),
    StateNotExist(u64)
}

#[derive(Clone, Debug)]
pub enum StateExecutionError {
    TransactionExecutionError(TxHash, TxExecutionError)
}

#[derive(Clone, Debug)]
pub enum TxExecutionError {
    SenderHasNotEnoughBalance,
    SenderHasNoAccount,
    NonceError(u64, u64),
}