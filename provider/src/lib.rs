pub mod state;
pub mod error;

use std::sync::Arc;

pub use database::traits::Database;
use primitives::{types::{Account, Address}};

use crate::{error::ProviderError, state::ExecutableState};

#[derive(Debug, Clone)]
pub struct ProviderFactory<DB: Database> {
    db: DB
}

impl<DB: Database + Clone> ProviderFactory<DB> {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    pub fn latest(&self) -> Provider<DB> {
        let block_no = self.db.block_number();
        self.state_by_block_number(block_no)
    }

    fn state_by_block_number(&self, block_no: u64) -> Provider<DB> {
        Provider{
            db: self.db.clone(),
            block_no: block_no
        }
    }
}

pub struct Provider<DB: Database> {
    db: DB,
    block_no: u64,
}

impl<DB: Database> Provider<DB> {
    pub fn basic_account(&self, address: Address) -> Result<Option<Account>, Box<dyn std::error::Error>>  {
        Ok(self.db.basic(&address)?)
    }

    pub fn executable_state(&self) -> Result<ExecutableState, ProviderError> {
        let (accounts_base, field_base) = match self.db.get_state(self.block_no) {
            Ok((account, field)) => (account, field),
            Err(e) => return Err(ProviderError::DatabaseError(e))
        };

        if accounts_base.is_none() || field_base.is_none() {
            return Err(ProviderError::StateNotExist(self.block_no));
        }
        let accounts_write = accounts_base.clone().unwrap();
        let accounts_base = Arc::new(accounts_base.unwrap());

        let field_write = field_base.clone().unwrap();
        let field_base = Arc::new(field_base.unwrap());

        Ok(ExecutableState {
            accounts_base, accounts_write, field_base, field_write
        })
    }
}