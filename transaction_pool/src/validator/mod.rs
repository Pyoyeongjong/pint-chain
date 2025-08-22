use std::sync::Arc;

use provider::{state::State, Database, ProviderFactory};

pub mod validtx;

#[derive(Debug)]
pub struct Validator<DB: Database> {
    inner: Arc<ValidatorInner<DB>>,
}

impl<DB: Database> Validator<DB> {
    pub fn new(provier: ProviderFactory<DB>) -> Self {
        Self {
            inner: Arc::new(ValidatorInner::new(provier)),
        }
    }
}

#[derive(Debug)]
pub struct ValidatorInner<DB: Database> {
    provider: ProviderFactory<DB>,
    current_state: Option<State>,
}

impl<DB: Database> ValidatorInner<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            provider,
            current_state: None,
        }
    }
}
