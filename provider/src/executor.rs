use std::{convert::Infallible, sync::Arc};

use primitives::{block::Block, transaction::Recovered, types::{Account, TxHash, B256, U256}};

use crate::{error::ExecutionError, state::{ExecutableState}};

pub struct Executor {
    pub state: ExecutableState,
    pub receipts: Vec<Receipt>,
}

impl Executor {

    pub fn new(state: ExecutableState) -> Self {
        Self { state, receipts: Vec::new() }
    }

    pub fn state(&mut self) -> &mut ExecutableState {
        &mut self.state
    }
    pub fn execute_transaction(&mut self, tx: &Recovered) 
    -> Result<Receipt, Infallible>{
        let mut receipt = Receipt { tx_hash: tx.hash(), fee: 0, success: true, error: None };
        let fee = match self.state.execute_transaction(tx) {
            Ok(fee) => fee,
            Err(err) => {
                receipt.success = false;
                receipt.error = Some(ExecutionError::StateExecutionError(err));
                0
            }
        };
        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    // For validation external payload
    pub fn execute_block(&mut self, block: &Block) -> Result<(), ExecutionError> {
        let transactions = &block.body;
        let proposer = block.header().proposer;
        let mut fee_sum = U256::ZERO;
        for transaction in transactions.iter() {
            match self.execute_transaction(transaction) {
                Ok(receipt) => {
                    fee_sum += U256::from(receipt.fee);
                }
                Err(_e) => {
                    continue;
                }
            }
        }

        // update mining results
        match self.state().accounts_write.get_mut(&proposer) {
            Some(account) => {
                account.add_balance(fee_sum);
            }
            None => {
                let mut new_account = Account::default();
                new_account.add_balance(fee_sum);
                self.state().accounts_write.insert(proposer, new_account);
            }
        }

        Ok(())
    }

    pub fn calculate_state_root(&self) -> B256 {
        self.state.calculate_state_root()
    }
}



#[derive(Debug, Clone)]
pub struct Receipt {
    pub tx_hash: TxHash,
    pub fee: u128,
    pub success: bool,
    pub error: Option<ExecutionError>,
}