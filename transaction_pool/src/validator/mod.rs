use std::sync::Arc;

use provider::{state::State, Database, ProviderFactory};

pub mod validtx;

pub struct Validator<DB: Database> {
    inner: Arc<ValidatorInner<DB>>,
}

pub struct ValidatorInner<DB: Database> {
    provider: ProviderFactory<DB>,
    current_state: Option<State>,
}
