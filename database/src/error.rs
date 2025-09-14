use std::{error::Error, fmt::Display};


#[derive(Debug)]
pub enum DatabaseError {
    BlockEncodeError,
    DataNotExists,
    DBError
}

impl Error for DatabaseError {

}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::BlockEncodeError => write!(f, "failed to encode block"),
            DatabaseError::DataNotExists => write!(f, "requested data does not exist in database"),
            DatabaseError::DBError => write!(f, "database error"),
        }
    }
}