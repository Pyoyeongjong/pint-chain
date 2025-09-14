use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};

use libmdbx::{orm::{Database, DatabaseChart, Decodable, Encodable}, table, table_info};
use once_cell::sync::Lazy;
use primitives::{block::Block, transaction::SignedTransaction, types::{ Account, Address, BlockHash, TxHash, U256 }, world::World};

use crate::{error::DatabaseError, traits::DatabaseTrait};

pub type BlockNo = u64;

#[derive(Debug)]
pub struct DBAdress {
    pub block_no: BlockNo,
    pub address: Address,
}

impl DBAdress {
    pub fn new(addr: Address, bno: u64) -> Self{
        Self { address: addr, block_no: bno }
    }
}

impl Encodable for DBAdress {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        let mut raw: Vec<u8> = Vec::new();
        raw.extend_from_slice(&self.block_no.to_be_bytes());
        raw.extend_from_slice(&self.address.encode());
        raw
    }
}

impl Decodable for DBAdress {
    fn decode(b: &[u8]) -> anyhow::Result<Self> {
        let mut u64_raw = [0u8; 8];
        u64_raw.copy_from_slice(&b[0..8]);
        let bno = u64::from_be_bytes(u64_raw);
        let mut addr_raw = [0u8; 20];
        addr_raw.copy_from_slice(&b[8..28]);
        let address = Address::from_byte(addr_raw);
        Ok(DBAdress { address, block_no: bno })
    }
}

table!(
    /// State Table
    (Basic) DBAdress => Account
);

table!(
    /// Block Table
    (Blocks) BlockNo => Block
);

table!(
    /// BlockHash
    (BlockByHash) BlockHash => BlockNo
);

table!(
    /// World State Snapshot
    (States) BlockNo => World  
);

table!(
    /// Transactions
    (Transactions) TxHash => SignedTransaction
);


pub static TABLES: Lazy<Arc<DatabaseChart>> = 
    Lazy::new(|| Arc::new([
        table_info!(Basic),
        table_info!(Blocks),
        table_info!(States),
        table_info!(Transactions),
    ].into_iter().collect()));


#[derive(Clone, Debug)]
pub struct MDBX {
    inner: Arc<Database>
}

impl MDBX {
    pub fn new() -> Self {
        let path = Path::new("./data/data");
        let mut pathbuf = PathBuf::new();
        pathbuf.push(path);
        let db = Arc::new(libmdbx::orm::Database::create(
            Some(pathbuf), 
            &TABLES
        ).unwrap());

        let mdbx = MDBX {inner: db};

        let tx = mdbx.inner.begin_read().unwrap();
        let mut cursor = tx.cursor::<Blocks>().unwrap();
        let is_empty = cursor.first().unwrap().is_none();
        drop(cursor);
        drop(tx);

        if is_empty {
            let genesis_block = Block::genesis_block();

            let tx = mdbx.inner.begin_readwrite().unwrap();
            {
                let mut cursor = tx.cursor::<Blocks>().unwrap();
                cursor.upsert(0, genesis_block).unwrap();
            }
            {
                let mut cursor = tx.cursor::<States>().unwrap();
                cursor.upsert(0, World::new()).unwrap();
            }
            tx.commit().unwrap();

            println!("MDBX: Genesis block initialized.");
        } else {
            println!("MDBX: DB already initialized, skipping genesis.");
        }

        mdbx
    }

    pub fn genesis_state() -> Self {
        let mut db = Self::new();
        let account = Account {
            nonce: 0,
            balance: U256::from(1000000000),
        };
        let address = Address::from_hex("28dcb1338b900419cd613a8fb273ae36e7ec2b1d".to_string()).unwrap();
        db.add_account(address, account.clone()).unwrap();
        let address = Address::from_hex("0534501c34f5a0f3fa43dc5d78e619be7edfa21a".to_string()).unwrap();
        db.add_account(address, account.clone()).unwrap();
        db
    }

    pub fn add_account(&mut self, address: Address, account: Account) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.inner.begin_readwrite().map_err(|_| DatabaseError::DBError)?;
        let mut cursor = tx.cursor::<Basic>().map_err(|_| DatabaseError::DBError)?;
        cursor.upsert(DBAdress::new(address, 0), account).map_err(|_| DatabaseError::DBError)?;
        tx.commit().unwrap();
        Ok(())
    }
}

impl DatabaseTrait for MDBX {
    fn latest_block_number(&self) -> u64 {
        let tx = self.inner.begin_read().unwrap();
        let mut cursor = tx.cursor::<Blocks>().unwrap();

        if let Some((block_no, _)) = cursor.last().unwrap() {
            block_no
        } else {
            0
        }
    }

    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        let tx = self.inner.begin_read().map_err(|_| DatabaseError::DBError)?;
        let latest = self.latest_block_number();
        let db_addr = DBAdress::new(*address, latest);
        match tx.get::<Basic>(db_addr) {
            Ok(res) => {
                Ok(res)
            }
            Err(_e) => return Err(Box::new(DatabaseError::DBError)),
        }
    }

    fn get_state(&self, block_no: u64) -> Result<(Option<std::collections::HashMap<Address, Account>>, Option<World>), Box<dyn std::error::Error>> {
        let tx = self.inner.begin_read().map_err(|_| DatabaseError::DBError)?;
        let cursor = tx.cursor::<Basic>().map_err(|_| DatabaseError::DBError)?;
        let mut accounts = HashMap::new();

        let start_key = DBAdress::new(Address::min(), block_no);

        let mut iter = cursor.walk(Some(start_key));

        while let Some(Ok((key, account))) = iter.next() {

            if key.block_no == block_no {
                accounts.insert(key.address, account);
            }
            else {
                break;
            }
        }
        let world = tx.get::<States>(block_no).map_err(|_| DatabaseError::DBError)?;
        Ok((Some(accounts), world))
    }

    fn get_block(&self, block_no: u64) -> Result<Block, Box<dyn std::error::Error>> {
        let tx = self.inner.begin_read().map_err(|_| DatabaseError::DBError)?;
        match tx.get::<Blocks>(block_no) {
            Ok(res) => {
                match res {
                    Some(block) => return Ok(block),
                    None => Err(Box::new(DatabaseError::DataNotExists)),
                }
            }
            Err(_e) => {
                return Err(Box::new(DatabaseError::DBError));
            }
        }

    }

    fn get_header(&self, block_no: u64) -> Result<primitives::block::Header, Box<dyn std::error::Error>> {

        match self.get_block(block_no) {
            Ok(block) => {
                Ok(block.header().clone())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn update(&self, new_account_state: std::collections::HashMap<Address, Account>, new_field_state: World, new_block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let latest_bno = self.latest_block_number();
        let new_latest = latest_bno + 1;

        let tx = self.inner.begin_readwrite().map_err(|_| DatabaseError::DBError)?;
        let mut cursor = tx.cursor::<Basic>().map_err(|_| DatabaseError::DBError)?;
        for (address, account) in new_account_state.iter() {

            cursor.upsert(DBAdress::new(*address, new_latest), *account).map_err(|_| DatabaseError::DBError)?;
        }

        let mut cursor = tx.cursor::<States>().map_err(|_| DatabaseError::DBError)?;
        cursor.upsert(new_latest, new_field_state).map_err(|_| DatabaseError::DBError)?;

        let mut cursor = tx.cursor::<Blocks>().map_err(|_| DatabaseError::DBError)?;
        cursor.upsert(new_latest, new_block).map_err(|_| DatabaseError::DBError)?;
        tx.commit().unwrap();
        println!("DB updated {}", new_latest);
        Ok(())
    }

    fn get_latest_block_header(&self) -> primitives::block::Header {
        let latest_bno = self.latest_block_number();
        self.get_header(latest_bno).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::mdbx::MDBX;


    #[test]
    fn test_db() {
        let mdbx = MDBX::genesis_state();
        dbg!(mdbx);
    }
}