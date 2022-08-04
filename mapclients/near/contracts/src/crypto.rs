use std::convert::TryFrom;
use std::ops::Sub;
use near_sdk::env;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json;
use crate::types::istanbul::{IstanbulAggregatedSeal, IstanbulMsg, G1_PUBLIC_KEY_LENGTH};
use crate::types::header::Hash;
use num_bigint::{BigInt as Integer, BigInt, Sign};
use num_traits::{Num, One};
use crate::serialization::rlp::big_int_to_rlp_compat_bytes;


const ALT_BN128_REGISTER: u64 = 1;
pub const REGISTER_EXPECTED_ERR: &str =
    "Register was expected to have data because we just wrote it into it.";

const ORDER: &str = "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";
const PRIME: &str = "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct G1 {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub x: [u8; 32],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub y: [u8; 32],
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

    pub fn from_le_slice(s: &[u8]) -> Result<Self, ()> {
        if let 64 = s.len() {
            let mut x = [0 as u8; 32];
            x.copy_from_slice(&s[..32]);
            x.reverse();

            let mut y = [0 as u8; 32];
            y.copy_from_slice(&s[32..]);
            y.reverse();

            return Ok(G1 { x, y });
        }

        Err(())
    }

    pub fn neg(&self) -> Self {
        let y = Integer::from_bytes_be(Sign::Plus, self.y.as_slice());
        let prime = Integer::from_bytes_be(Sign::Plus, hex::decode(PRIME).unwrap().as_slice());
        let neg_y = prime.sub(&y);

        let mut neg_y_bytes = integer_to_vec_32(&neg_y, true);

        Self {
            x: self.x,
            y: <[u8; 32]>::try_from(neg_y_bytes).unwrap(),
        }
    }
}

fn integer_to_vec_32(i: &BigInt, be: bool) -> Vec<u8> {
    let mut bytes: Vec<u8> = if be { i.to_signed_bytes_be() } else { i.to_signed_bytes_le() };
    if bytes.len() < 32 {
        let mut res: Vec<u8> = vec![0; 32 - bytes.len()];
        if be {
            res.append(&mut bytes);
            res
        } else {
            bytes.append(&mut res);
            bytes
        }
    } else {
        bytes
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

pub fn sum_points<'a>(points: &'a Vec<G1>, bitmap: &'a Integer) -> Result<G1, &'a str> {
    let filtered: Vec<Vec<u8>> = points
        .iter()
        .enumerate()
        .filter(|(i, _)| bitmap.bit(*i as _))
        .map(|(_, v)| [&[0], to_le_bytes(&v.x).as_ref(), to_le_bytes(&v.y).as_ref()].concat())
        .collect();

    if filtered.len() == 0 {
        return Err("no g1 point to sum");
    } else if filtered.len() == 1 {
        let slice = filtered[0].as_slice();
        return Ok(G1::from_le_slice(&slice[1..]).expect(format!("{:?}", filtered).as_str()));
    }

    let buf = filtered.concat();

    let res = map_bn128::alt_bn128_g1_sum(buf).unwrap();
    // unsafe {
    //     near_sys::alt_bn128_g1_sum(buf.len() as _, buf.as_ptr() as _, ALT_BN128_REGISTER);
    // }

    // let res = env::read_register(ALT_BN128_REGISTER).expect(REGISTER_EXPECTED_ERR);
    // if res.len() != G1_PUBLIC_KEY_LENGTH {
    //     return Err("alt_bn128_g1_sum get invalid result");
    // }

    Ok(G1::from_le_slice(res.as_slice()).unwrap())
}

pub fn check_aggregated_g2_pub_key(points: &Vec<G1>, bitmap: &Integer, agg_g2_pk: &G2) -> bool {
    let g1_pk_sum = sum_points(points, bitmap).unwrap();
    let g2 = get_g2();
    let g1 = get_g1();
    let buf = pack_points(&g1_pk_sum, &g2, &g1.neg(), agg_g2_pk);

    map_bn128::alt_bn128_pairing_check(buf).unwrap()
    // let mut res = 0;
    // unsafe {
    //     res = near_sys::alt_bn128_pairing_check(buf.len() as _, buf.as_ptr() as _);
    // }

    // res == 1
}

pub fn check_sealed_signature(agg_seal: &IstanbulAggregatedSeal, hash: &Hash, agg_g2_pk: &G2) -> bool {
    let sig_on_g1 = G1::from_slice(agg_seal.signature.as_slice()).unwrap();
    let g2 = get_g2();
    let proposal_seal = prepare_commited_seal(*hash, &agg_seal.round);
    let hash_to_g1 = hash_to_g1(proposal_seal);

    let buf = pack_points(&sig_on_g1, &g2, &hash_to_g1.neg(), agg_g2_pk);

    map_bn128::alt_bn128_pairing_check(buf).unwrap()
    // let mut res = 0;
    // unsafe {
    //     res = near_sys::alt_bn128_pairing_check(buf.len() as _, buf.as_ptr() as _);
    // }

    // res == 1
}

fn hash_to_g1(msg: Vec<u8>) -> G1 {
    let h = Integer::from_bytes_be(Sign::Plus, env::keccak256(msg.as_slice()).as_slice());
    let scalar = h.modpow(&Integer::one(), &Integer::from_str_radix(ORDER, 16).unwrap());
    let g1 = get_g1();

    let buf: Vec<u8> = [
        &to_le_bytes(&g1.x),
        &to_le_bytes(&g1.y),
        integer_to_vec_32(&scalar, false).as_slice()].concat();

    let res = map_bn128::alt_bn128_g1_multiexp(buf).unwrap();
    // unsafe {
    //     near_sys::alt_bn128_g1_multiexp(buf.len() as _, buf.as_ptr() as _, ALT_BN128_REGISTER);
    // }

    // let res = env::read_register(ALT_BN128_REGISTER).expect(REGISTER_EXPECTED_ERR);
    // assert_eq!(64, res.len(), "G1 multiexp get invalid result");

    G1::from_le_slice(res.as_slice()).unwrap()
}

fn pack_points(p0: &G1, p1: &G2, p2: &G1, p3: &G2) -> Vec<u8> {
    [
        to_le_bytes(&p0.x),
        to_le_bytes(&p0.y),
        to_le_bytes(&p1.xr),
        to_le_bytes(&p1.xi),
        to_le_bytes(&p1.yr),
        to_le_bytes(&p1.yi),
        to_le_bytes(&p2.x),
        to_le_bytes(&p2.y),
        to_le_bytes(&p3.xr),
        to_le_bytes(&p3.xi),
        to_le_bytes(&p3.yr),
        to_le_bytes(&p3.yi)
    ].concat()
}

fn to_le_bytes(bytes: &[u8; 32]) -> [u8; 32] {
    let mut buf = [0; 32];

    for (k, v) in bytes.iter().enumerate() {
        buf[32 - k - 1] = *v;
    }

    buf
}

fn prepare_commited_seal(hash: Hash, round: &Integer) -> Vec<u8> {
    let round_bytes = big_int_to_rlp_compat_bytes(&round);
    let commit_bytes = [IstanbulMsg::Commit as u8];

    [&hash[..], &round_bytes[..], &commit_bytes[..]].concat()
}