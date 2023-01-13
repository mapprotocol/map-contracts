extern crate core;

mod types;
use std::collections::HashSet;
pub use types::*;
mod serialization;
pub use serialization::*;
mod crypto;
mod hash;
mod macros;
pub mod traits;

use crate::crypto::{check_aggregated_g2_pub_key, check_sealed_signature};
use crate::types::header::Address;
use crate::types::proof::{verify_trie_proof, ReceiptProof};
use crate::types::{header::Header, istanbul::get_epoch_number, istanbul::IstanbulExtra};
pub use crypto::{G1, G2, REGISTER_EXPECTED_ERR};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::env::keccak256;
use near_sdk::json_types::{Base64VecU8, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, serde_json, AccountId, Gas, PanicOnDefault};
use num::cast::ToPrimitive;
use num_bigint::BigInt as Integer;

const ECDSA_SIG_LENGTH: usize = 65;
const ECDSA_REGISTER: u64 = 2;
const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(15_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MapLightClient {
    epoch_records: UnorderedMap<u64, EpochRecord>,
    epoch_size: u64,
    header_height: u64,
    owner: AccountId,
    max_records: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MapLightClientV1 {
    epoch_records: UnorderedMap<u64, EpochRecord>,
    epoch_size: u64,
    header_height: u64,
    owner: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EpochRecord {
    pub threshold: U64,
    pub epoch: U64,
    pub validators: Vec<Validator>,
}

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    g1_pub_key: G1,
    weight: U64,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    address: Address,
}

#[near_bindgen]
impl MapLightClient {
    #[init]
    pub fn new(
        threshold: U64,
        validators: Vec<Validator>,
        epoch: U64,
        epoch_size: U64,
        owner: AccountId,
    ) -> Self {
        assert!(!Self::initialized(), "already initialized");
        assert_ne!(0, validators.len(), "empty validators!");
        assert_ne!(0, threshold.0, "threashold should not be 0");

        let mut addresses: HashSet<Address> = HashSet::default();
        let mut total_weight = 0;
        for validator in validators.iter() {
            assert_ne!(
                0,
                validator.weight.0,
                "the weight of validator {} is 0",
                serde_json::to_string(&validator.address).unwrap()
            );
            addresses.insert(validator.address);
            total_weight += validator.weight.0;
        }
        assert_eq!(
            validators.len(),
            addresses.len(),
            "duplicated address in validators"
        );
        assert!(
            threshold.0 <= total_weight,
            "threashold should not greater than validators' total weight"
        );

        let mut val_records = UnorderedMap::new(b"v".to_vec());
        val_records.insert(
            &epoch.0,
            &EpochRecord {
                threshold,
                epoch,
                validators,
            },
        );

        let max_records = 30 * 3 * 24 * 60 * 60 * 2 / 9 / epoch_size.0; // keep records for 3 months

        Self {
            epoch_records: val_records,
            epoch_size: epoch_size.into(),
            header_height: (epoch.0 - 1) * epoch_size.0,
            owner,
            max_records,
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate_client() -> Self {
        let oldClient: MapLightClientV1 =
            env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        let max_records = 30 * 3 * 24 * 60 * 60 * 2 / 9 / oldClient.epoch_size;
        Self {
            epoch_records: oldClient.epoch_records,
            epoch_size: oldClient.epoch_size,
            header_height: oldClient.header_height,
            owner: oldClient.owner,
            max_records,
        }
    }

    pub fn initialized() -> bool {
        env::state_read::<MapLightClient>().is_some()
    }

    pub fn update_block_header(&mut self, header: &Header, agg_pk: G2) {
        let block_num = header.number.to_u64().unwrap();
        let block_exp = self.header_height + self.epoch_size;
        assert_eq!(block_exp, block_num, "block header height is incorrect");

        // check ecdsa and bls signature
        let epoch = get_epoch_number(block_num, self.epoch_size as u64);
        let mut extra = IstanbulExtra::from_rlp(&header.extra).unwrap();
        let cur_epoch_record = &self.epoch_records.get(&epoch).unwrap();
        self.verify_signatures(header, agg_pk, &extra, cur_epoch_record);

        // update validators' pair keys
        self.update_next_validators(cur_epoch_record, &mut extra);

        self.header_height = block_num;

        log!(
            "block header {} is updated for the next epoch {} by {}",
            block_num,
            epoch + 1,
            env::signer_account_id()
        )
    }

    pub fn verify_proof_data(&self, receipt_proof: ReceiptProof) {
        let header = &receipt_proof.header;
        let extra = IstanbulExtra::from_rlp(&header.extra).unwrap();

        // check ecdsa and bls signature
        let epoch = get_epoch_number(header.number.to_u64().unwrap(), self.epoch_size as u64);
        let epoch_record = &self.epoch_records.get(&epoch).unwrap_or_else(|| {
            let range = self.get_verifiable_header_range();
            panic!(
                "cannot get epoch record for block {}, expected range[{}, {}]",
                header.number.to_string(),
                range.0 .0,
                range.1 .0
            )
        });
        self.verify_signatures(header, receipt_proof.agg_pk, &extra, epoch_record);

        // Verify receipt included into header
        let data = verify_trie_proof(
            header.receipt_hash,
            receipt_proof.key_index,
            receipt_proof.proof,
        );

        let receipt_data = receipt_proof.receipt.encode_index();

        assert_eq!(
            hex::encode(receipt_data),
            hex::encode(data),
            "receipt data is not equal to the value in trie"
        );
    }

    pub fn get_verifiable_header_range(&self) -> (U64, U64) {
        let count = self.epoch_records.len() * self.epoch_size;
        let begin = self.header_height + self.epoch_size + 1 - count;
        let end = self.header_height + self.epoch_size;
        (begin.into(), end.into())
    }

    pub fn get_header_height(&self) -> U64 {
        self.header_height.into()
    }

    pub fn get_epoch_size(&self) -> U64 {
        self.epoch_size.into()
    }

    pub fn get_record_for_epoch(&self, epoch: U64) -> Option<EpochRecord> {
        self.epoch_records.get(&epoch.0)
    }

    fn verify_signatures(
        &self,
        header: &Header,
        agg_pk: G2,
        extra: &IstanbulExtra,
        epoch_record: &EpochRecord,
    ) {
        let addresses = epoch_record.validators.iter().map(|x| x.address).collect();
        // check ecdsa signature
        self.verify_ecdsa_signature(header, &extra.seal, &addresses);

        // check agg seal
        self.verify_aggregated_seal(header, extra, epoch_record, &agg_pk);
    }

    fn is_quorum(&self, bitmap: &Integer, validators: &Vec<Validator>, threshold: u64) -> bool {
        let weight: u64 = validators
            .iter()
            .enumerate()
            .filter(|(i, _)| bitmap.bit(*i as u64))
            .map(|(_, v)| v.weight.0)
            .sum();

        weight >= threshold
    }

    fn verify_ecdsa_signature(
        &self,
        header: &Header,
        signature: &Vec<u8>,
        addresses: &Vec<Address>,
    ) {
        assert_eq!(
            ECDSA_SIG_LENGTH,
            signature.len(),
            "invalid ecdsa signature length"
        );

        let res = addresses
            .iter()
            .filter(|x| x.as_slice() == header.coinbase.as_slice())
            .count();
        assert_eq!(1, res, "the header's coinbase is not in validators");

        let v = signature.last().unwrap();
        let header_hash = header.hash_without_seal().unwrap();
        let hash = keccak256(header_hash.as_slice());
        let res;

        unsafe {
            res = near_sys::ecrecover(
                hash.len() as _,
                hash.as_ptr() as _,
                (signature.len() - 1) as _,
                signature.as_ptr() as _,
                *v as _,
                0,
                ECDSA_REGISTER,
            );
        }

        assert_ne!(0, res, "ecrecover returns 0");

        let res = env::read_register(ECDSA_REGISTER).expect(REGISTER_EXPECTED_ERR);
        assert_eq!(
            64,
            res.len(),
            "the length of the ecrecover result is not expected"
        );

        let pub_key_hash = env::keccak256(res.as_slice());
        assert_eq!(
            &header.coinbase,
            &pub_key_hash[12..],
            "ecdsa signer is not correct"
        );
    }

    fn verify_aggregated_seal(
        &self,
        header: &Header,
        extra: &IstanbulExtra,
        epoch_record: &EpochRecord,
        agg_g2_pk: &G2,
    ) {
        let pair_keys = epoch_record
            .validators
            .iter()
            .map(|x| x.g1_pub_key)
            .collect();
        assert!(
            self.is_quorum(
                &extra.aggregated_seal.bitmap,
                &epoch_record.validators,
                epoch_record.threshold.into()
            ),
            "threshold is not satisfied"
        );

        assert!(
            check_aggregated_g2_pub_key(&pair_keys, &extra.aggregated_seal.bitmap, agg_g2_pk),
            "check g2 pub key failed"
        );

        let header_hash = header.hash().unwrap();
        assert!(
            check_sealed_signature(&extra.aggregated_seal, &header_hash, agg_g2_pk),
            "check sealed signature failed"
        )
    }

    fn update_next_validators(
        &mut self,
        cur_epoch_record: &EpochRecord,
        extra: &mut IstanbulExtra,
    ) {
        let mut validator_list: Vec<Validator> = cur_epoch_record
            .validators
            .iter()
            .enumerate()
            .filter(|(i, _)| !extra.removed_validators.bit(*i as _))
            .map(|(_, v)| *v)
            .collect();

        let mut added_validators: Vec<Validator> = extra
            .added_g1_public_keys
            .iter()
            .zip(extra.added_validators.iter())
            .map(|(g1_key, address)| Validator {
                g1_pub_key: G1::from_slice(g1_key).unwrap(),
                weight: U64(1),
                address: *address,
            })
            .collect();

        validator_list.append(&mut added_validators);

        let total_weight: u64 = validator_list.iter().map(|x| x.weight.0).sum();

        let next_epoch = cur_epoch_record.epoch.0 + 1;

        let next_epoch_record = EpochRecord {
            epoch: U64(next_epoch),
            validators: validator_list,
            threshold: U64(total_weight - total_weight / 3),
        };

        log!(
            "epoch {} validators: remove: {}, add: {:?}, total: {:?}",
            next_epoch,
            extra.removed_validators,
            serde_json::to_string(&extra.added_validators).unwrap(),
            serde_json::to_string(&next_epoch_record).unwrap()
        );

        self.epoch_records.insert(&next_epoch, &next_epoch_record);
        if next_epoch >= self.max_records {
            let epoch_to_remove = next_epoch - self.max_records;
            self.epoch_records.remove(&epoch_to_remove);
        }
    }

    pub fn update_owner(&mut self, new_owner: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        self.owner = new_owner;
    }

    pub fn upgrade_client(&mut self, code: Base64VecU8) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        let current_id = env::current_account_id();
        let promise_id = env::promise_batch_create(&current_id);
        env::promise_batch_action_deploy_contract(promise_id, &code.0);
        env::promise_batch_action_function_call(
            promise_id,
            "migrate_client",
            &[],
            0,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY,
        );
    }
}
