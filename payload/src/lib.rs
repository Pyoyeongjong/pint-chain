use provider::{Database, ProviderFactory};
use transaction_pool::Pool;

pub struct PayloadBuilder<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
}