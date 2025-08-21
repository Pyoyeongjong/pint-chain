use std::sync::Arc;

use provider::Database;

use crate::pool::PoolInner;

pub mod pool;
pub mod validator;
pub mod identifier;
pub mod ordering;

pub struct Pool<DB: Database> {
    inner: Arc<PoolInner<DB>>,
}

