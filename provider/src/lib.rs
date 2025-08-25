pub mod state;
pub use database::traits::Database;
use primitives::{block, types::{Account, Address}};

#[derive(Debug, Clone)]
pub struct ProviderFactory<DB: Database> {
    db: DB
}

impl<DB: Database + Clone> ProviderFactory<DB> {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    pub fn latest(&self) -> Result<Provider<DB>, Box<dyn std::error::Error>> {
        let block_no = self.db.block_number();
        self.state_by_block_number(block_no)
    }

    fn state_by_block_number(&self, block_no: u64) -> Result<Provider<DB>, Box<dyn std::error::Error>> {
        Ok(Provider{
            db: self.db.clone(),
            block_no: block_no
        })
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
}