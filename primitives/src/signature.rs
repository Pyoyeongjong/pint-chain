use crate::types::U256;

/// ESDCA Signature
#[derive(Debug, Clone)]
pub struct Signature {
    pub y_parity: bool,
    pub r: U256,
    pub s: U256,
}