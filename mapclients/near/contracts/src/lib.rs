extern crate core;

mod types;
mod serialization;
mod crypto;
mod macros;
mod traits;

use std::ops::Add;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, PanicOnDefault};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};
use uint::construct_uint;
use crypto::{G1, G2, sum_points, REGISTER_EXPECTED_ERR};
use crate::types::{istanbul::IstanbulExtra, istanbul::get_epoch_number, header::Header};
use crate::types::{errors::Kind, header::Address};
use num::cast::ToPrimitive;
use num_bigint::BigInt as Integer;
use crate::crypto::{check_aggregated_g2_pub_key, check_sealed_signature};
use crate::traits::FromBytes;
use crate::types::header::{Bloom, Hash};
use crate::types::proof::{ReceiptProof, verify_trie_proof};

const ECDSA_SIG_LENGTH: usize = 65;
const ECDSA_REGISTER: u64 = 2;
const MAX_RECORD: u64 = 20;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MapLightClient {
    val_records: UnorderedMap<u64, Validators>,
    epoch_size: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Validators {
    threshold: u128,
    epoch: u64,
    validator_list: Vec<Validator>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Validator {
    g1_pub_key: G1,
    weight: u128,
    address: Address,
}

#[near_bindgen]
impl MapLightClient {
    #[init]
    pub fn new(threshold: u128,
               pair_keys: Vec<G1>,
               addresses: Vec<Address>,
               weights: Vec<u128>, epoch: u64, epoch_size: u64) -> Self {
        assert!(!Self::initialized(), "Already initialized");
        assert_eq!(pair_keys.len(), addresses.len(), "lengths of pair keys and addresses are not equal");
        assert_eq!(pair_keys.len(), weights.len(), "lengths of pair keys and weights are not equal");

        let mut validator_list = Vec::new();
        for (i, key) in pair_keys.into_iter().enumerate() {
            validator_list.push(Validator {
                g1_pub_key: key,
                weight: weights[i],
                address: addresses[i],
            })
        }

        let mut val_records = UnorderedMap::new(b"v".to_vec());
        val_records.insert(&epoch, &Validators {
            threshold,
            epoch,
            validator_list,
        });

        Self {
            val_records,
            epoch_size,
        }
    }

    pub fn initialized() -> bool {
        env::state_read::<MapLightClient>().is_some()
    }

    pub fn update_block_header(&mut self, header: &Header, agg_pk: G2) {
        let block_num = header.number.to_u64().unwrap();
        assert_eq!(0, block_num % self.epoch_size, "Header number is incorrect");

        let mut extra = IstanbulExtra::from_rlp(&header.extra).unwrap();

        // check ecdsa and bls signature
        let epoch = get_epoch_number(header.number.to_u64().unwrap(), self.epoch_size as u64);
        let validators = &self.val_records.get(&epoch).unwrap();
        self.verify_signatures(header, agg_pk, &extra, &validators);

        // update validators' pair keys
        self.update_next_validators(validators, &mut extra, epoch + 1);
    }

    pub fn verify_proof_data(&self, receipt_proof: ReceiptProof) {
        let header = &receipt_proof.header;
        let extra = IstanbulExtra::from_rlp(&header.extra).unwrap();

        // check ecdsa and bls signature
        let epoch = get_epoch_number(header.number.to_u64().unwrap(), self.epoch_size as u64);
        let validators = &self.val_records.get(&epoch).unwrap();
        self.verify_signatures(header, receipt_proof.agg_pk, &extra, validators);

        // Verify receipt included into header
        let data =
            verify_trie_proof(header.receipt_hash, receipt_proof.key_index, receipt_proof.proof);

        let receipt_data = receipt_proof.receipt.encode_index();

        assert_eq!(hex::encode(receipt_data), hex::encode(data), "receipt data is not equal to the value in trie");
    }

    fn verify_signatures(&self, header: &Header, agg_pk: G2, extra: &IstanbulExtra, validators: &Validators) {
        let addresses = validators.validator_list
            .iter()
            .map(|x| x.address)
            .collect();
        // check ecdsa signature
        self.verify_ecdsa_signature(header, &extra.seal, &addresses);

        // check agg seal
        self.verify_aggregated_seal(header, &extra, validators, &agg_pk);
    }

    fn is_quorum(&self, bitmap: &Integer, weights: &Vec<u128>, threshold: u128) -> bool {
        let weight: u128 = weights
            .iter()
            .enumerate()
            .filter(|(i, _)| bitmap.bit(*i as u64))
            .map(|(k, v)| v)
            .sum();

        return weight >= threshold;
    }

    fn is_quorum_default_wight(&self, bitmap: &Integer, pair_keys: &Vec<G1>, threshold: u128) -> bool {
        let mut weight = 0;
        for (i, _) in pair_keys.iter().enumerate() {
            if bitmap.bit(i as u64) {
                weight += 1;
            }
        }

        return weight >= threshold;
    }

    fn verify_ecdsa_signature(&self, header: &Header, signature: &Vec<u8>, addresses: &Vec<Address>) {
        assert_eq!(ECDSA_SIG_LENGTH, signature.len(), "invalid ecdsa signature length");

        let res: Vec<Address> = addresses.iter().filter(|x| x.as_slice() == header.coinbase.as_slice()).cloned().collect();
        assert_eq!(1, res.len(), "the header's coinbase is not in validators");

        let v = signature.last().unwrap();
        let header_hash = header.hash_without_seal().unwrap();
        let mut res = 0;

        unsafe {
            res = near_sys::ecrecover(header_hash.len() as _,
                                      header_hash.as_ptr() as _,
                                      (signature.len() - 1) as _,
                                      signature.as_ptr() as _,
                                      *v as _,
                                      0,
                                      ECDSA_REGISTER);
        }

        let res = env::read_register(ECDSA_REGISTER).expect(REGISTER_EXPECTED_ERR);
        assert_eq!(64, res.len(), "read register after ecrecover get invalid result");

        let pub_key_hash = env::keccak256(res.as_slice());
        assert_eq!(&header.coinbase, &pub_key_hash[12..], "ecdsa signer is not correct");
    }

    fn verify_aggregated_seal(&self, header: &Header, extra: &IstanbulExtra, validators: &Validators, agg_g2_pk: &G2) {
        let pair_keys = validators.validator_list
            .iter()
            .map(|x| x.g1_pub_key)
            .collect();
        assert!(self.is_quorum_default_wight(&extra.aggregated_seal.bitmap, &pair_keys, validators.threshold), "threshold is not satisfied");

        assert!(check_aggregated_g2_pub_key(&pair_keys, &extra.aggregated_seal.bitmap, agg_g2_pk), "check g2 pub key failed");

        let header_hash = header.hash().unwrap();
        assert!(check_sealed_signature(&extra.aggregated_seal, &header_hash, agg_g2_pk), "check sealed signature failed")
    }

    fn update_next_validators(&mut self, validators: &Validators, extra: &mut IstanbulExtra, epoch: u64) {
        let mut validator_list: Vec<Validator> = validators.validator_list
            .iter()
            .enumerate()
            .filter(|(i, _)| !extra.removed_validators.bit(*i as _))
            .map(|(_, v)| *v)
            .collect();

        let mut added_validators: Vec<Validator> = extra.added_g1_public_keys
            .iter()
            .zip(extra.added_validators.iter())
            .map(|(g1_key, address)| Validator {
                g1_pub_key: G1::from_slice(g1_key).unwrap(),
                // TODO: update later?
                weight: 1,
                address: *address,
            })
            .collect();

        validator_list.append(&mut added_validators);

        let total_weight :u128 = validator_list
            .iter()
            .map(|x| x.weight)
            .sum();

        let next_validators = Validators {
            epoch,
            validator_list,
            threshold: total_weight - total_weight / 3,
        };

        log!("epoch {} validators: {:?}", epoch, next_validators);

        self.val_records.insert(&epoch, &next_validators);
        if epoch >= MAX_RECORD {
            let epoch_to_remove = epoch - MAX_RECORD;
            self.val_records.remove(&epoch_to_remove);
        }
    }
}