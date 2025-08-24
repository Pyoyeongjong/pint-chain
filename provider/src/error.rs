use database::error::DatabaseError;

#[derive(Debug)]
pub enum ProviderError{
    DatabaseError(DatabaseError),
}

impl From<DatabaseError> for ProviderError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(value)
    }
}