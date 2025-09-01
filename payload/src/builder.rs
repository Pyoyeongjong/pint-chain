use primitives::{block::Header, types::{Address, BlockHash, PayloadId}};

#[derive(Default)]
pub struct BuildArguments {
    pub address: Address,
    pub parent_header: Header,
    pub attributes: PayloadBuilderAttributes,
}

impl BuildArguments {
    pub fn noob(address: Address) -> Self {
        let mut res = Self::default();
        res.address = address;
        res.attributes.max_transactions = 20;
        res
    }
}

#[derive(Default)]
pub struct PayloadBuilderAttributes {
    pub id: PayloadId,
    pub parent_hash: BlockHash,
    pub next_difficulty: u32,
    pub max_transactions: u32,
}