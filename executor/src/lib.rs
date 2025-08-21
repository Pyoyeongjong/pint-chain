use primitives::types::TxHash;
use provider::state::ExecutableState;

use crate::error::ExecutionError;

pub mod error;

pub struct Executor {
    state: ExecutableState,
    receipts: Vec<Receipt>,
}

pub struct Receipt {
    tx_hash: TxHash,
    success: bool,
    error: Option<ExecutionError>,
}