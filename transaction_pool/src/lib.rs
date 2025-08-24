use std::sync::Arc;

use provider::{Database, ProviderFactory};

use crate::pool::PoolInner;

pub mod pool;
pub mod validator;
pub mod identifier;
pub mod ordering;
pub mod mock;
pub mod error;

#[derive(Debug, Clone)]
pub struct Pool<DB: Database> {
    pool: Arc<PoolInner<DB>>,
}

impl<DB: Database> Pool<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            pool: Arc::new(PoolInner::new(provider)),
        }
    }

    pub fn inner(&self) -> &PoolInner<DB> {
        &self.pool
    }
}


