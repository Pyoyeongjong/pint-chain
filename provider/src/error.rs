use primitives::{error::RecoveryError, types::TxHash};

#[derive(Clone, Debug)]
pub enum ExecutionError {
    StateExecutionError(StateExecutionError),
    TransactionRecoveryError(RecoveryError),
    TotalFeeisDifferent,
}

#[derive(Debug)]
pub enum ProviderError {
    DatabaseError(Box<dyn std::error::Error>),
    ExecutionError(ExecutionError),
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