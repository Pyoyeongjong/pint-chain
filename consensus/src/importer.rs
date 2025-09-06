use primitives::{block::{Block, BlockValidationResult}, error::{BlockImportError, BlockValidatioError}};
use provider::{executor::Executor, Database, ProviderFactory};

#[derive(Debug)]
pub struct BlockImporter<DB: Database> {
    provider: ProviderFactory<DB>
}

impl<DB: Database> BlockImporter<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self { provider }
    }

    pub fn import_new_block(&self, block: Block) -> Result<(), BlockImportError> {
        if block.header.height != self.provider.block_number() + 1 {
            return Err(BlockImportError::BlockHeightError);
        }   
        let res = self.validate_block(&block)?;
        if res.success {
            if let Err(_e) = self.provider.import_new_block(block) {
                return Err(BlockImportError::ProviderError);
            }            
        }

        Ok(())
    }

    fn validate_block(&self, block: &Block) -> Result<BlockValidationResult, BlockImportError> {
        // validate block with no state
        let mut result: BlockValidationResult = Self::validate_block_with_no_state(&block)?;

        let state_provider = self.provider.latest();
        let executable_state = match state_provider.executable_state() {
            Ok(exec_state) => exec_state,
            Err(_e) => return Err(BlockImportError::ProviderError),
        };

        let mut executor = Executor::new(executable_state);

        // validate block with state
        match executor.execute_block(&block) {
            Ok((_, _)) => {
                result.success();
            }

            Err(_e) => {
                result.failed();
                result.add_error(BlockValidatioError::ExecutionError);
            }
        }
        
        Ok(result)
    }

    fn validate_block_with_no_state(_block: &Block) -> Result<BlockValidationResult, BlockImportError>{
        // todo!
        Ok(BlockValidationResult {
            success: true,
            error: None,
        })
    }
}
