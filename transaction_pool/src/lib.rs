use std::sync::Arc;

use provider::{Database, ProviderFactory};

use crate::pool::PoolInner;

pub mod pool;
pub mod validator;
pub mod identifier;
pub mod ordering;

#[derive(Debug, Clone)]
pub struct Pool<DB: Database> {
    inner: Arc<PoolInner<DB>>,
}

impl<DB: Database> Pool<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            inner: Arc::new(PoolInner::new(provider)),
        }
    }
}

