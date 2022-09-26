mod serialization;
mod traits;
mod macros;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, env, Gas, near_bindgen, PanicOnDefault, Promise};
use num_bigint::BigInt as Integer;
use hex::FromHex;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::Deserializer};
use near_sdk::serde::de::Error;
use crate::traits::FromBytes;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MapLightClient {
    value: bool,
}

/// HASH_LENGTH represents the number of bytes used in a header hash
pub const HASH_LENGTH: usize = 32;

/// ADDRESS_LENGTH represents the number of bytes used in a header Ethereum account address
pub const ADDRESS_LENGTH: usize = 20;

/// BLOOM_BYTE_LENGTH represents the number of bytes used in a header log bloom
pub const BLOOM_BYTE_LENGTH: usize = 256;

/// BLOCK_NONCE_LENGTH represents the number of bytes used in a header nonce
pub const NONCE_LENGTH: usize = 8;

/// Hash is the output of the cryptographic digest function
pub type Hash = [u8; HASH_LENGTH];

/// Address represents the 20 byte address of an Ethereum account
pub type Address = [u8; ADDRESS_LENGTH];

/// Bloom represents a 2048 bit bloom filter
pub type Bloom = [u8; BLOOM_BYTE_LENGTH];

/// Nonce represents a 64 bit nonce
pub type Nonce = [u8; NONCE_LENGTH];

const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(15_000_000_000_000);

/// Header contains block metadata in Celo Blockchain
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde", rename_all = "camelCase")]
pub struct Header {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub parent_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub coinbase: Address,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub root: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub tx_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub receipt_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub bloom: Bloom,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub number: Integer,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub gas_limit: Integer,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub gas_used: Integer,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub time: Integer,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub extra: Vec<u8>,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub min_digest: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub nonce: Nonce,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub base_fee: Integer,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct G2 {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub xr: [u8; 32],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub xi: [u8; 32],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub yr: [u8; 32],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub yi: [u8; 32],
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiptProof {
    pub header: Header,
    pub agg_pk: G2,
    pub receipt: Receipt,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub key_index: Vec<u8>,
    pub proof: Vec<ProofEntry>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Receipt {
    pub receipt_type: u128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub post_state_or_status: Vec<u8>,
    pub cumulative_gas_used: u64,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LogEntry {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub address: Address,
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub topics: Vec<Hash>,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ProofEntry (Vec<u8>);

impl Serialize for ProofEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let hex_string = hex::encode(self.0.as_slice());
        if hex_string.is_empty() {
            return serializer.serialize_str("");
        }

        serializer.serialize_str(&(String::from("0x") + &hex_string))
    }
}

impl<'de> Deserialize<'de> for ProofEntry {
    fn deserialize< D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        if  !s.starts_with("0x") {
            return Err(Error::custom("should start with 0x"));
        }

        let data = Vec::from_hex(&s[2..]).map_err(|err| {
            Error::custom(err.to_string())
        })?;
        Ok(ProofEntry{0: data})
    }
}

impl FromBytes for Bloom {
    fn from_bytes(data: &[u8]) -> Result<&Bloom, ()> {
        slice_as_array_ref!(&data[..BLOOM_BYTE_LENGTH], BLOOM_BYTE_LENGTH)
    }
}

impl FromBytes for Address {
    fn from_bytes(data: &[u8]) -> Result<&Address, ()> {
        slice_as_array_ref!(&data[..ADDRESS_LENGTH], ADDRESS_LENGTH)
    }
}

impl FromBytes for Nonce {
    fn from_bytes(data: &[u8]) -> Result<&Nonce, ()> {
        slice_as_array_ref!(&data[..NONCE_LENGTH], NONCE_LENGTH)
    }
}

impl FromBytes for Hash {
    fn from_bytes(data: &[u8]) -> Result<&Hash, ()> {
        slice_as_array_ref!(&data[..HASH_LENGTH], HASH_LENGTH)
    }
}

#[near_bindgen]
impl MapLightClient {
    #[init]
    pub fn new() -> Self {
        Self {
            value: true
        }
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old struct with different name to deserialize it first).
    /// After migrate goes live on MainNet, return this implementation for next updates.
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        // let this: MapLightClient = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        // this
        Self {
            value: true
        }
    }

    pub fn initialized() -> bool {
        env::state_read::<MapLightClient>().is_some()
    }

    pub fn update_block_header(&mut self, header: &Header, agg_pk: G2) {

    }

    pub fn verify_proof_data(&self, receipt_proof: ReceiptProof) {
        assert!(self.value);
    }

    pub fn upgrade_self(&mut self, code: Base64VecU8) {
        let current_id = env::current_account_id();
        let promise_id = env::promise_batch_create(&current_id);
        env::promise_batch_action_deploy_contract(promise_id, &code.0);
        env::promise_batch_action_function_call(
            promise_id,
            "migrate",
            &[],
            0,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY,
        );
    }

    pub fn delete_self(beneficiary: AccountId){
        Promise::new(env::current_account_id())
            .delete_account(beneficiary);
    }
}