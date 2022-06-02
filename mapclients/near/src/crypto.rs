use near_sdk::env;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};
use near_sdk::serde_json;
use near_sys::panic;
use crate::Kind;
use crate::istanbul::min_quorum_size;
use crate::types::istanbul::{IstanbulExtra, IstanbulAggregatedSeal, IstanbulMsg};
use crate::types::header::{Header, Hash};
use num_bigint::{BigInt as Integer, Sign};
use num_traits::{Num, One};
use crate::Validators;
use crate::serialization::bytes::hexstring;
use crate::serialization::rlp::{
    big_int_to_rlp_compat_bytes, rlp_field_from_bytes, rlp_to_big_int,
};


const ALT_BN128_REGISTER: u64 = 1;
pub const REGISTER_EXPECTED_ERR: &str =
    "Register was expected to have data because we just wrote it into it.";

const order: &str = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct G1 {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub x: [u8; 32],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub y: [u8; 32],
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
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

impl G1 {
    pub fn from_slice(s: &[u8]) -> Result<Self, ()> {
        if let 64 = s.len() {
            let mut x = [0 as u8; 32];
            x.copy_from_slice(&s[..32]);

            let mut y = [0 as u8; 32];
            y.copy_from_slice(&s[32..]);

            return Ok(G1 { x, y });
        }

        Err(())
    }
}

fn get_g1() -> G1 {
    serde_json::from_str("{\"x\":\"0x0000000000000000000000000000000000000000000000000000000000000001\",\
                                         \"y\":\"0x0000000000000000000000000000000000000000000000000000000000000002\"}").unwrap()
}

fn get_g2() -> G2 {
    serde_json::from_str("{\
    \"xr\":\"0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\",\
    \"xi\":\"0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\",\
    \"yr\":\"0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa\",\
    \"yi\":\"0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\"\
    }"
    ).unwrap()
}

pub fn check_bit(bits: &Vec<u8>, index: usize) -> bool {
    let a = bits.get(index / 8).expect(format!("index {} out of range", index).as_str());
    return a & (1 << (index % 8)) != 0;
}

pub fn sum_points(points: &Vec<G1>, bitmap: &Integer) -> G1 {
    let filtered: Vec<Vec<u8>> = points
        .iter()
        .enumerate()
        .filter(|(i, _)| bitmap.bit(*i as _))
        .map(|(k, v)| [&[0], v.x.as_ref(), v.y.as_ref()].concat())
        .collect();

    let buf = filtered.concat();

    unsafe {
        near_sys::alt_bn128_g1_sum(buf.len() as _, buf.as_ptr() as _, ALT_BN128_REGISTER);
    }

    let res = env::read_register(ALT_BN128_REGISTER).expect(REGISTER_EXPECTED_ERR);
    assert_eq!(64, res.len(), "sum G1 point get invalid result");

    G1::from_slice(res.as_slice()).unwrap()
}

pub fn check_aggregated_g2_pub_key(points: &Vec<G1>, bitmap: &Integer, agg_g2_pk: &G2) -> bool {
    let g1_pk_sum = sum_points(points, bitmap);
    let g2 = get_g2();
    let g1 = get_g1();
    let buf = pack_points(&g1_pk_sum, &g2, &g1, agg_g2_pk);

    let mut res = 0;
    unsafe {
        res = near_sys::alt_bn128_pairing_check(buf.len() as _, buf.as_ptr() as _);
    }

    res == 1
}

pub fn check_sealed_signature(agg_seal: &IstanbulAggregatedSeal, hash: &Hash, agg_g2_pk: &G2) -> bool {
    let sig_on_g1 = G1::from_slice(agg_seal.signature.as_slice()).unwrap();
    let g2 = get_g2();
    let proposal_seal = prepare_commited_seal(*hash, &agg_seal.round);
    let hash_to_g1 = hash_to_g1(proposal_seal);

    let buf = pack_points(&sig_on_g1, &g2, &hash_to_g1, agg_g2_pk);

    let mut res = 0;
    unsafe {
        res = near_sys::alt_bn128_pairing_check(buf.len() as _, buf.as_ptr() as _);
    }

    res == 1
}

fn hash_to_g1(msg: Vec<u8>) -> G1 {
    let h = Integer::from_bytes_be(Sign::Plus, env::keccak256(msg.as_slice()).as_slice());
    let scalar = h.modpow(&Integer::one(), &Integer::from_str_radix(order, 16).unwrap());
    let g1 = get_g1();

    let buf: Vec<u8> = [&g1.x, &g1.y, scalar.to_signed_bytes_be().as_slice()].concat();

    unsafe {
        near_sys::alt_bn128_g1_multiexp(buf.len() as _, buf.as_ptr() as _, ALT_BN128_REGISTER);
    }

    let res = env::read_register(ALT_BN128_REGISTER).expect(REGISTER_EXPECTED_ERR);
    assert_eq!(64, res.len(), "G1 multiexp get invalid result");

    G1::from_slice(res.as_slice()).unwrap()
}

fn pack_points(p0: &G1, p1: &G2, p2: &G1, p3: &G2) -> Vec<u8> {
    [p0.x, p0.y, p1.xr, p1.xi, p1.yr, p1.yi, p2.x, p2.y, p3.xr, p3.xi, p3.yr, p3.yi].concat()
}

fn prepare_commited_seal(hash: Hash, round: &Integer) -> Vec<u8> {
    let round_bytes = big_int_to_rlp_compat_bytes(&round);
    let commit_bytes = [IstanbulMsg::Commit as u8];

    [&hash[..], &round_bytes[..], &commit_bytes[..]].concat()
}