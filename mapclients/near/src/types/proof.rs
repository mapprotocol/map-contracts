use crate::crypto::G2;
use rlp::{Rlp, Encodable, RlpStream};
use crate::types::header::{Header, Address, Bloom, Hash};
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};

/*
    struct receiptProof {
        blockHeader header;
        G2 aggPk;
        txReceipt receipt;
        bytes keyIndex;
        bytes[] proof;
    }


    struct txReceipt{
        uint256  receiptType;
        bytes   postStateOrStatus;
        uint256   cumulativeGasUsed;
        bytes bloom;
        txLog[] logs;
    }



    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }
 */

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiptProof {
    pub header: Header,
    pub agg_pk: G2,
    pub receipt: Receipt,
    pub key_index: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Receipt {
    pub receipt_type: u128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub post_state_or_status: Vec<u8>,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub cumulative_gas_used: [u8; 256],
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LogEntry {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub address: Address,
    pub topics: Vec<Hash>,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub data: Vec<u8>,
}

impl Receipt {
    pub fn encode_index(&self) -> Vec<u8> {
        if let 0 = self.receipt_type {
            return rlp::encode(self);
        }

        let mut res: Vec<u8> = Vec::new();
        res.push(self.receipt_type as _);
        res.append(&mut rlp::encode(self));
        res
    }
}

impl Encodable for Receipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);

        // post_state_or_status
        s.append(&self.post_state_or_status);

        // cumulative_gas_used
        s.append(&self.cumulative_gas_used.as_ref());

        // bloom
        s.append(&self.bloom.as_ref());

        // logs
        s.append_list::<LogEntry, _>(&self.logs);
    }
}

impl Encodable for LogEntry {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);

        // address
        s.append(&self.address.as_ref());

        let topics: Vec<Vec<u8>> = self.topics.iter().map(|t| t.to_vec()).collect();

        // topics
        s.append_list::<Vec<u8>, _>(&topics);

        // data
        s.append(&self.data);
    }
}

pub fn verify_trie_proof(expected_root: Hash, key: Vec<u8>, proof: Vec<Vec<u8>>) -> Vec<u8> {
    let mut actual_key = vec![];
    for el in key {
        actual_key.push(el / 16);
        actual_key.push(el % 16);
    }
    _verify_trie_proof(expected_root.to_vec(), &actual_key, &proof, 0, 0)
}

fn _verify_trie_proof(
    expected_root: Vec<u8>,
    key: &Vec<u8>,
    proof: &Vec<Vec<u8>>,
    key_index: usize,
    proof_index: usize,
) -> Vec<u8> {
    let node = &proof[proof_index];

    if key_index == 0 {
        // trie root is always a hash
        assert_eq!(near_keccak256(node), expected_root.as_slice());
    } else if node.len() < 32 {
        // if rlp < 32 bytes, then it is not hashed
        assert_eq!(node.as_slice(), expected_root);
    } else {
        assert_eq!(near_keccak256(node), expected_root.as_slice());
    }

    let node = Rlp::new(&node.as_slice());

    if node.iter().count() == 17 {
        // Branch node
        if key_index == key.len() {
            assert_eq!(proof_index + 1, proof.len());
            get_vec(&node, 16)
        } else {
            let new_expected_root = get_vec(&node, key[key_index] as usize);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + 1,
                proof_index + 1,
            )
        }
    } else {
        // Leaf or extension node
        assert_eq!(node.iter().count(), 2);
        let path_u8 = get_vec(&node, 0);
        // Extract first nibble
        let head = path_u8[0] / 16;
        // assert!(0 <= head); is implicit because of type limits
        assert!(head <= 3);

        // Extract path
        let mut path = vec![];
        if head % 2 == 1 {
            path.push(path_u8[0] % 16);
        }
        for val in path_u8.into_iter().skip(1) {
            path.push(val / 16);
            path.push(val % 16);
        }
        assert_eq!(path.as_slice(), &key[key_index..key_index + path.len()]);

        if head >= 2 {
            // Leaf node
            assert_eq!(proof_index + 1, proof.len());
            assert_eq!(key_index + path.len(), key.len());
            get_vec(&node, 1)
        } else {
            // Extension node
            let new_expected_root = get_vec(&node, 1);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + path.len(),
                proof_index + 1,
            )
        }
    }
}

pub fn near_keccak256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&near_sdk::env::keccak256(data).as_slice());
    buffer
}

/// Get element at position `pos` from rlp encoded data,
/// and decode it as vector of bytes
fn get_vec(data: &Rlp, pos: usize) -> Vec<u8> {
    data.at(pos).unwrap().as_val::<Vec<u8>>().unwrap()
}