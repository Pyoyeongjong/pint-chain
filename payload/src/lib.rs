use provider::{Database, ProviderFactory};
use transaction_pool::Pool;

#[derive(Clone, Debug)]
pub struct PayloadBuilder<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
}

impl<DB: Database> PayloadBuilder<DB> {
    pub fn new(provider: ProviderFactory<DB>, pool: Pool<DB>) -> Self {
        Self {
            provider,
            pool
        }
    }
}