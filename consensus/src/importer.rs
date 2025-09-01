use primitives::{block::{Block, BlockValidationResult}, error::{BlockImportError, BlockValidatioError}};
use provider::{executor::Executor, state, Database, ProviderFactory};

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
        let mut result: BlockValidationResult = Self::validate_block_with_no_state(&block)?;

        let state_provider = self.provider.latest();
        let executable_state = match state_provider.executable_state() {
            Ok(exec_state) => exec_state,
            Err(e) => return Err(BlockImportError::ProviderError),
        };

        let mut executor = Executor::new(executable_state);

        // validate block with state
        match executor.execute_block(&block) {
            Ok(()) => {
                result.success();
            }

            Err(_e) => {
                result.failed();
                result.add_error(BlockValidatioError::ExecutionError);
            }
        }
        
        Ok(result)
    }

    fn validate_block_with_no_state(block: &Block) -> Result<BlockValidationResult, BlockImportError>{
        // todo!
        Ok(BlockValidationResult {
            success: true,
            error: None,
        })
    }
}
