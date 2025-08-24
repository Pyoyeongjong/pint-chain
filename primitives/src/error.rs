use std::array::TryFromSliceError;

pub enum BlockImportError {
    NoopImporter,
}

pub enum EncodeError{
    Invalid,
}

#[derive(Debug)]
pub enum DecodeError{
    TooShortRawData(Vec<u8>),
    InvalidAddress(AddressError),
    InvalidSignature(SignatureError),
    TryFromSliceError(TryFromSliceError),
}

impl From<TryFromSliceError> for DecodeError {
    fn from(err: TryFromSliceError) -> Self {
        Self::TryFromSliceError(err)
    }
}

#[derive(Debug)]
pub enum AddressError {
    FromHexError(hex::FromHexError),
    InvalidLength(usize),
}

impl From<hex::FromHexError> for AddressError {
    fn from(err: hex::FromHexError) -> Self {
        Self::FromHexError(err)
    }
}

#[derive(Debug)]
pub enum SignatureError {
    InvalidParity(u64),
}

#[derive(Debug)]
pub enum RecoveryError {
    RecIdError,
    RecKeyError,
    AddressError(AddressError),
    HashGetError,
    RecoveryFromDigestError,
}

impl From<AddressError> for RecoveryError {
    fn from(err: AddressError) -> Self {
        Self::AddressError(err)
    }
}