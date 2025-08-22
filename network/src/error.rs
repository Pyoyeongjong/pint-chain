use std::io::Error;

#[derive(Debug)]
pub enum NetworkStartError {
    LinstenerBindingError(Error),
}
