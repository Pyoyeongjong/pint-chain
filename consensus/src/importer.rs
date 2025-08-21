use provider::{Database, ProviderFactory};

pub struct BlockImporter<DB: Database> {
    provider: ProviderFactory<DB>
}