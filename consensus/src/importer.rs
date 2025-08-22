use provider::{Database, ProviderFactory};

#[derive(Debug)]
pub struct BlockImporter<DB: Database> {
    provider: ProviderFactory<DB>
}

impl<DB: Database> BlockImporter<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self { provider }
    }
}