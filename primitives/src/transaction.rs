use std::ops::Add;

use k256::ecdsa::RecoveryId;
use k256::ecdsa::VerifyingKey;
use k256::EncodedPoint;
use sha2::Sha256;
use sha2::Digest;
use crate::error::DecodeError;
use crate::error::RecoveryError;
use crate::{signature::Signature, types::{Address, ChainId, TxHash, U256, B256}};

pub trait Tx {
    fn chain_id(&self) -> ChainId;
    fn nonce(&self) -> u64;
    fn to(&self) -> Address;
    fn fee(&self) -> u128;
    fn value(&self) -> U256;
}

/// Raw Transaction
#[derive(Debug, Clone)]
pub struct Transaction {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub to: Address,
    pub fee: u128,
    pub value: U256,
}

impl Transaction {
    pub fn encode_for_signing(&self) -> TxHash {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.to_string().as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        hasher.update(self.to.get_addr());
        hasher.update(self.fee.to_string().as_bytes());
        hasher.update(self.value.to_string().as_bytes());
        B256::from_slice(&hasher.finalize())
    }

    fn encode(&self) -> Vec<u8> {
        let mut arr: [u8; 84] = [0u8; 84];
        arr[0..8].copy_from_slice(&self.chain_id.to_be_bytes());
        arr[8..16].copy_from_slice(&self.nonce.to_be_bytes());
        arr[16..36].copy_from_slice(&self.to.get_addr());
        arr[36..52].copy_from_slice(&self.fee.to_be_bytes());
        arr[52..].copy_from_slice(&self.value.to_be_bytes::<32>());
        arr.to_vec()
    }

    pub fn raw_decode(data: &Vec<u8>) -> Result<(Self, usize), DecodeError> {
        let raw: [u8; 84] = data[0..84].try_into()?;
        let chain_id: ChainId = ChainId::from_be_bytes(raw[0..8].try_into()?);
        let nonce: u64 = u64::from_be_bytes(raw[8..16].try_into()?);
        let to = match Address::from_hex(hex::encode(&raw[16..36])) {
            Ok(addr) => addr,
            Err(e) => return Err(DecodeError::InvalidAddress(e)),
        };
        let fee: u128 = u128::from_be_bytes(raw[36..52].try_into()?);
        let value: U256 = U256::from_be_bytes::<32>(raw[52..84].try_into()?);

        Ok((
            Self {
                chain_id,
                nonce,
                to,
                fee,
                value,
            },
            84 as usize,
        ))
    }

    fn into_signed(self, signature: Signature) -> SignedTransaction {
        let hash = self.encode_for_signing();
        SignedTransaction { tx: self, signature, hash }
    }
}

impl Tx for Transaction {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn to(&self) -> Address {
        self.to
    }

    fn fee(&self) -> u128 {
        self.fee
    }

    fn value(&self) -> U256 {
        self.value
    }
}

/// Transaction with Signature
#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Signature,
    pub hash: TxHash,
}

impl SignedTransaction {

    pub fn transaction(&self) -> &Transaction {
        &self.tx
    }

    pub fn new(tx: Transaction, signature: Signature, hash: TxHash) -> Self{
        Self { tx, signature, hash }
    }

    pub fn encode(&self) -> Vec<u8> {
        let tx_arr = self.tx.encode();
        let sig_arr = self.signature.as_bytes().to_vec();
        [tx_arr, sig_arr].concat()
    }

    pub fn decode(raw: &Vec<u8>) -> Result<(Self, usize), crate::error::DecodeError> {
        let size = raw.len();
        let (tx, tx_size) = Transaction::raw_decode(&raw)?;

        if size < tx_size + 65 {
            return Err(DecodeError::TooShortRawData(raw.clone()));
        }

        let sig_raw: [u8; 65] = match raw[tx_size..tx_size + 65].try_into() {
            Ok(arr) => arr,
            Err(err) => return Err(DecodeError::TryFromSliceError(err)),
        };

        let signature = Signature::raw_decode(&sig_raw)?;
        let signed = tx.into_signed(signature);
        Ok((signed, size))
    }

    pub fn recover_signer(&self) -> Result<Address, RecoveryError> {
        let y_parity: u8 = if self.signature.y_parity() { 1 } else { 0 };
        let recid = RecoveryId::from_byte(y_parity).unwrap(); // safe!
        let signature: k256::ecdsa::Signature = self.signature.clone().into();
        let hash = self.hash;

        let recovered_key = match VerifyingKey::recover_from_digest(
            Sha256::new_with_prefix(hash),
            &signature,
            recid,
        ) {
            Ok(key) => key,
            Err(_) => return Err(RecoveryError::RecoveryFromDigestError),
        };

        let recovered_pubkey_uncompressed: EncodedPoint = recovered_key.to_encoded_point(false);
        let recovered_pubkey_bytes = recovered_pubkey_uncompressed.as_bytes();
        let recovered_address: [u8; 20] = recovered_pubkey_bytes
            [recovered_pubkey_bytes.len() - 20..]
            .try_into()
            .expect("slice is not 20 bytes");

        Ok(Address::from_byte(recovered_address))
    }

    pub fn into_recovered(self) -> Result<Recovered, RecoveryError> {
        let signer = self.recover_signer()?;
        Ok(Recovered {
            tx: self, signer

        })
    }
}

impl Tx for SignedTransaction {
    fn chain_id(&self) -> ChainId {
        self.transaction().chain_id()
    }

    fn nonce(&self) -> u64 {
        self.transaction().nonce()
    }

    fn to(&self) -> Address {
        self.transaction().to()
    }

    fn fee(&self) -> u128 {
        self.transaction().fee()
    }

    fn value(&self) -> U256 {
        self.transaction().value()
    }
}

#[derive(Debug, Clone)]
// SignedTransaction -> Recovered
pub struct Recovered {
    tx: SignedTransaction,
    signer: Address,
}

impl Recovered {
    pub fn tx(&self) -> &SignedTransaction {
        &self.tx
    }

    pub fn signer(&self) -> Address {
        self.signer
    }

    pub fn hash(&self) -> TxHash {
        self.tx().hash
    }
}

impl Tx for Recovered {
    fn chain_id(&self) -> ChainId {
        self.tx().chain_id()
    }

    fn nonce(&self) -> u64 {
        self.tx().nonce()
    }

    fn to(&self) -> Address {
        self.tx().to()
    }

    fn fee(&self) -> u128 {
        self.tx().fee()
    }

    fn value(&self) -> U256 {
        self.tx().value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::{ecdsa::{RecoveryId, Signature as ECDSASig, SigningKey}, EncodedPoint};
    use sha2::{Digest, Sha256};

    fn create_key_pairs(seed: &[u8]) -> (SigningKey, Vec<u8>) {
        let private_key_random = Sha256::digest(&seed);
        let signing_key = SigningKey::from_bytes(&private_key_random).unwrap();

        let verifying_key = signing_key.clone().verifying_key().clone();
        let pubkey_uncompressed: EncodedPoint = verifying_key.to_encoded_point(false);
        let pubkey_bytes = pubkey_uncompressed.as_bytes();
        let address = pubkey_bytes[pubkey_bytes.len() - 20..].to_vec();
        (signing_key, address)
    }

    #[test]
    fn test_primitives_encode_and_decode_transaction() {
        let (signing_key, sender) = create_key_pairs("chain".as_bytes());
        let sender = Address::from_byte(sender.try_into().unwrap());
        dbg!(&sender.get_addr_hex());

        let (_, receiver) = create_key_pairs("banana".as_bytes());
        let receiver = Address::from_byte(receiver.try_into().unwrap());
        dbg!(receiver.get_addr_hex());

        let tx = Transaction {
            chain_id: 0,
            nonce: 0,
            to: receiver,
            fee: 5,
            value: U256::from(1000),
        };

        let tx_hash = tx.encode_for_signing();
        let digest = Sha256::new_with_prefix(tx_hash);
        let (sig, recid): (ECDSASig, RecoveryId) =
            signing_key.sign_digest_recoverable(digest).unwrap();
        let signature = Signature::from_sig(sig, recid);

        let signed_tx = SignedTransaction::new(tx, signature, tx_hash);
        let encoded = signed_tx.encode();
        dbg!(hex::encode(&encoded));

        let (recovered_signed, _) = SignedTransaction::decode(&encoded).unwrap();
        let recovered_sender = recovered_signed.recover_signer().unwrap();

        assert_eq!(sender, recovered_sender);
    }
}

