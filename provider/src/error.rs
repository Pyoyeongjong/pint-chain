use database::error::DatabaseError;
use primitives::types::TxHash;

pub enum ProviderError {
    DatabaseError(DatabaseError),
    StateNotExist(u64)
}

pub enum ExecutionError {
    TransactionExecutionError(TxHash, TransactionExecError)
}

pub enum TransactionExecError {
    SenderHasNotEnoughBalance,
    SenderHasNoAccount,
    NonceError,
}