pub mod state;
pub mod world;

use std::sync::Arc;
pub use database::traits::Database;

use crate::state::State;


pub struct ProviderFactory<DB: Database> {
    db: Arc<DB>
}

pub struct Provider {
    state: State,
    block_no: u64,
}

