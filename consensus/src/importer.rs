use primitives::{block::{Block, BlockValidationResult}, error::BlockImportError};
use provider::{state, Database, ProviderFactory};

#[derive(Debug)]
pub struct BlockImporter<DB: Database> {
    provider: ProviderFactory<DB>
}

impl<DB: Database> BlockImporter<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self { provider }
    }

    pub fn validate_block(&self, block: Block) -> Result<BlockValidationResult, BlockImportError> {
        // validate block with no state
        let _res = Self::validate_block_with_no_state(&block);

        let state_provider = self.provider.latest();
        let mut executable_state = match state_provider.executable_state() {
            Ok(exec_state) => exec_state,
            Err(e) => return Err(BlockImportError::ProviderError),
        };

        // validate block with state
        let res = executable_state.execute_block(&block);

        
        Ok(BlockValidationResult {})
    }

    fn validate_block_with_no_state(block: &Block) -> Result<(), BlockImportError>{
        todo!()
    }
}
