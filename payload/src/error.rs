use provider::error::ProviderError;

pub enum PayloadBuilderError {
    ProviderError(ProviderError)
}

impl From<ProviderError> for PayloadBuilderError {
    fn from(value: ProviderError) -> Self {
        Self::ProviderError(value)
    }
}