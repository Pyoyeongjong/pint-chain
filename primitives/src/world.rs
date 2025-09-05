use alloy_primitives::B256;

#[derive(Debug, Clone)]
pub struct World {}

impl World {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn calculate_hash(&self) -> B256 {
        B256::default()
    }

    pub fn encode(&self) -> Vec<u8> {
        Vec::with_capacity(32)
    }
}