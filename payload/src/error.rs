use provider::error::ProviderError;

#[derive(Debug)]
pub enum PayloadBuilderError {
    ProviderError(ProviderError)
}

impl From<ProviderError> for PayloadBuilderError {
    fn from(value: ProviderError) -> Self {
        Self::ProviderError(value)
    }
}