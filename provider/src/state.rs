use std::{collections::HashMap, sync::Arc};

use primitives::{block::Block, transaction::{self, Recovered, SignedTransaction, Tx}, types::{Account, Address, TxHash, U256}, world::World};

use crate::error::{ExecutionError, TransactionExecError};


#[derive(Debug)]
pub struct State {
    accounts: Arc<HashMap<Address, Account>>,
    field: Arc<World>,
}

pub struct ExecutableState {
    pub accounts_base: Arc<HashMap<Address, Account>>,
    pub accounts_write: HashMap<Address, Account>,
    pub field_base: Arc<World>,
    pub field_write: World,
}

impl ExecutableState {
    pub fn execute_block(&mut self, block: &Block) -> Result<(), ExecutionError> {
        let transactions = &block.body;
        let proposer = block.header().proposer;
        let mut fee_sum = U256::ZERO;
        for transaction in transactions.iter() {
            match self.execute_transaction(transaction) {
                Ok(fee) => {
                    fee_sum += U256::from(fee)
                }
                Err(e) => return Err(ExecutionError::TransactionExecutionError(transaction.hash(), e))
            }
        }

        match self.accounts_write.get_mut(&proposer) {
            Some(account) => {
                account.add_balance(fee_sum);
            }
            None => {
                let mut new_account = Account::default();
                new_account.add_balance(fee_sum);
                self.accounts_write.insert(proposer, new_account);
            }
        }

        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Recovered) -> Result<u128, TransactionExecError> {
        let sender = transaction.signer();
        let receiver = transaction.to();

        let mut sender_account = match self.accounts_write.get(&sender) {
            Some(account) => account.clone(),
            // sender must have balance because of fee 
            None => return Err(TransactionExecError::SenderHasNoAccount)
        };

        let mut receiver_account = match self.accounts_write.get(&receiver) {
            Some(account) => account.clone(),
            None => {
                Account::default()
            }
        };

        if U256::from(transaction.fee()) > sender_account.balance() - transaction.value() {
            return Err(TransactionExecError::SenderHasNotEnoughBalance);
        }

        if sender_account.nonce() != transaction.nonce() {
            return Err(TransactionExecError::NonceError);
        }

        sender_account.sub_balance(transaction.value());
        sender_account.sub_balance(U256::from(transaction.fee()));
        receiver_account.add_balance(transaction.value());
        sender_account.increase_nonce();

        self.accounts_write.insert(sender, sender_account);
        self.accounts_write.insert(receiver, receiver_account);

        // TODO: Update World.

        Ok(transaction.fee())
    }
}