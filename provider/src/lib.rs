pub mod state;
pub mod world;

pub use database::traits::Database;

use crate::state::State;

#[derive(Debug, Clone)]
pub struct ProviderFactory<DB: Database> {
    db: DB
}

impl<DB: Database> ProviderFactory<DB> {
    pub fn new(db: DB) -> Self {
        Self { db }
    }
}

pub struct Provider {
    state: State,
    block_no: u64,
}

