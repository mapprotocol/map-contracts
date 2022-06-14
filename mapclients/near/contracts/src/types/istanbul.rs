use std::borrow::Borrow;
use crate::types::errors::{ Kind};
use crate::serialization::rlp::{
    big_int_to_rlp_compat_bytes, rlp_field_from_bytes, rlp_to_big_int,
};
use crate::{Header, slice_as_array_ref};
use crate::traits::{DefaultFrom, FromBytes};
use crate::types::header::Address;
use num_bigint::BigInt as Integer;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};

/// PUBLIC_KEY_LENGTH represents the number of bytes used to represent BLS public key
pub const PUBLIC_KEY_LENGTH: usize = 128;

/// SerializedPublicKey is a public key of a validator that is used to i.e sign the validator set
/// in the header
pub type SerializedPublicKey = [u8; PUBLIC_KEY_LENGTH];

/// G1_PUBLIC_KEY_LENGTH represents the number of bytes used to represent G1 BLS public key
pub const G1_PUBLIC_KEY_LENGTH: usize = 64;

/// SerializedPublicKey is a G1 public key of a validator that is used to i.e sign the validator set
/// in the header
pub type SerializedG1PublicKey = [u8; G1_PUBLIC_KEY_LENGTH];

/// ISTANBUL_EXTRA_VANITY_LENGTH represents the number of bytes used to represent validator vanity
pub const ISTANBUL_EXTRA_VANITY_LENGTH: usize = 32;

/// IstanbulExtraVanity is a portion of extra-data bytes reserved for validator vanity
pub type IstanbulExtraVanity = [u8; ISTANBUL_EXTRA_VANITY_LENGTH];

#[allow(dead_code)]
pub enum IstanbulMsg {
    PrePrepare,
    Prepare,
    Commit,
    RoundChange,
}

/// IstanbulAggregatedSeal contains the aggregated BLS signature created via IBFT consensus
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase",crate = "near_sdk::serde")]
pub struct IstanbulAggregatedSeal {
    /// Bitmap is a bitmap having an active bit for each validator that signed this block
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub bitmap: Integer,

    /// Signature is an aggregated BLS signature resulting from signatures by each validator that signed this block
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub signature: Vec<u8>,

    /// Round is the round in which the signature was created.
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub round: Integer,
}

impl IstanbulAggregatedSeal {
    pub fn new() -> Self {
        Self {
            bitmap: Integer::default(),
            signature: Vec::default(),
            round: Integer::default(),
        }
    }
}

impl Encodable for IstanbulAggregatedSeal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);

        // bitmap
        s.append(&big_int_to_rlp_compat_bytes(&self.bitmap));

        // signature
        s.append(&self.signature);

        // round
        s.append(&big_int_to_rlp_compat_bytes(&self.round));
    }
}

impl Decodable for IstanbulAggregatedSeal {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(IstanbulAggregatedSeal {
            bitmap: rlp_to_big_int(rlp, 0)?,
            signature: rlp.val_at(1)?,
            round: rlp_to_big_int(rlp, 2)?,
        })
    }
}

/// IstanbulExtra represents IBFT consensus state data
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase",crate = "near_sdk::serde")]
pub struct IstanbulExtra {
    /// The validators that have been added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_validators: Vec<Address>,

    /// The BLS public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_public_keys: Vec<SerializedPublicKey>,

    /// The G1 BLS public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_g1_public_keys: Vec<SerializedG1PublicKey>,

    /// Bitmap having an active bit for each removed validator in the block
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub removed_validators: Integer,

    /// ECDSA signature by the proposer
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub seal: Vec<u8>,

    /// Contains the aggregated BLS signature created via IBFT consensus
    pub aggregated_seal: IstanbulAggregatedSeal,

    /// Contains and aggregated BLS signature for the previous block
    pub parent_aggregated_seal: IstanbulAggregatedSeal,
}

impl IstanbulExtra {
    pub fn from_rlp(bytes: &[u8]) -> Result<Self, Kind> {
        if bytes.len() < ISTANBUL_EXTRA_VANITY_LENGTH {
            return Err(Kind::RlpDecodeError);
        }

        rlp::decode(&bytes[ISTANBUL_EXTRA_VANITY_LENGTH..])
            .map_err(|e| Kind::RlpDecodeError)
    }

    pub fn to_rlp(&self, vanity: &IstanbulExtraVanity) -> Vec<u8> {
        let payload = rlp::encode(self);

        [&vanity[..], &payload[..]].concat()
    }
}

impl Encodable for IstanbulExtra {
    fn rlp_append(&self, s: &mut RlpStream) {
        // added_validators
        s.begin_list(7);
        s.begin_list(self.added_validators.len());
        for address in self.added_validators.iter() {
            s.append(&address.as_ref());
        }

        // added_public_keys
        s.begin_list(self.added_public_keys.len());
        for address in self.added_public_keys.iter() {
            s.append(&address.as_ref());
        }

        // added_g1_public_keys
        s.begin_list(self.added_g1_public_keys.len());
        for address in self.added_g1_public_keys.iter() {
            s.append(&address.as_ref());
        }

        // removed_validators
        s.append(&big_int_to_rlp_compat_bytes(&self.removed_validators));

        // seal
        s.append(&self.seal);

        // aggregated_seal
        s.append(&self.aggregated_seal);

        // parent_aggregated_seal
        s.append(&self.parent_aggregated_seal);
    }
}

impl Decodable for IstanbulExtra {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let added_validators: Result<Vec<Address>, DecoderError> = rlp
            .at(0)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        let added_public_keys: Result<Vec<SerializedPublicKey>, DecoderError> = rlp
            .at(1)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        let added_g1_public_keys: Result<Vec<SerializedG1PublicKey>, DecoderError> = rlp
            .at(2)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        Ok(IstanbulExtra {
            added_validators: added_validators?,
            added_public_keys: added_public_keys?,
            added_g1_public_keys: added_g1_public_keys?,
            removed_validators: rlp_to_big_int(rlp, 3)?,
            seal: rlp.val_at(4)?,
            aggregated_seal: rlp.val_at(5)?,
            parent_aggregated_seal: rlp.val_at(6)?,
        })
    }
}

impl FromBytes for IstanbulExtraVanity {
    fn from_bytes(data: &[u8]) -> Result<&IstanbulExtraVanity, Kind> {
        slice_as_array_ref!(
            &data[..ISTANBUL_EXTRA_VANITY_LENGTH],
            ISTANBUL_EXTRA_VANITY_LENGTH
        )
    }
}

impl FromBytes for SerializedPublicKey {
    fn from_bytes(data: &[u8]) -> Result<&SerializedPublicKey, Kind> {
        slice_as_array_ref!(&data[..PUBLIC_KEY_LENGTH], PUBLIC_KEY_LENGTH)
    }
}

impl DefaultFrom for SerializedPublicKey {
    fn default() -> Self {
        [0; PUBLIC_KEY_LENGTH]
    }
}

impl FromBytes for SerializedG1PublicKey {
    fn from_bytes(data: &[u8]) -> Result<&SerializedG1PublicKey, Kind> {
        slice_as_array_ref!(&data[..G1_PUBLIC_KEY_LENGTH], G1_PUBLIC_KEY_LENGTH)
    }
}

impl DefaultFrom for SerializedG1PublicKey {
    fn default() -> Self {
        [0; G1_PUBLIC_KEY_LENGTH]
    }
}

// Retrieves the block number within an epoch. The return value will be 1-based.
// There is a special case if the number == 0. It is basically the last block of the 0th epoch,
// and should have a value of epoch_size
pub fn get_number_within_epoch(number: u64, epoch_size: u64) -> u64 {
    let number = number % epoch_size;
    if number == 0 {
        epoch_size
    } else {
        number
    }
}

pub fn get_epoch_number(number: u64, epoch_size: u64) -> u64 {
    let epoch_number = number / epoch_size;

    if is_last_block_of_epoch(number, epoch_size) {
        epoch_number
    } else {
        epoch_number + 1
    }
}

pub fn istanbul_filtered_header(header: &Header, keep_seal: bool) -> Result<Header, Kind> {
    let mut new_header = header.clone();

    let mut extra = IstanbulExtra::from_rlp(&new_header.extra)?;
    if !keep_seal {
        extra.seal = Vec::new();
    }
    extra.aggregated_seal = IstanbulAggregatedSeal::new();

    let payload = extra.to_rlp(IstanbulExtraVanity::from_bytes(&new_header.extra)?);
    new_header.extra = payload;

    Ok(new_header)
}

pub fn is_last_block_of_epoch(number: u64, epoch_size: u64) -> bool {
    get_number_within_epoch(number, epoch_size) == epoch_size
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::serde_json;
    use super::*;
    use num_traits::Num;
    use crate::Bloom;
    use crate::types::header::{HASH_LENGTH, NONCE_LENGTH};

    // tiny example to assert validity of basic data
    const ISTANBUL_EXTRA_TINY: &str = "f7ea9444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212c0c00c80c3808080c3808080";

    // random example
    const ISTANBUL_EXTRA_DUMPED: &str = "f901caea9444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212f90104b8800bb4595ae9b572777e51b9bcf510aaeb14ff679bf4ac6e5daa6f24913ea3184112aa4bfaf020de54956689c2124c40fa3f42554261ef58c85920fe8948ab82e60ada74eb113ef4b9f83ba1f4b6e9fc3592295a99a4ed319d840bfae5a70de6850518087f816ba55f0c4c86e5842b46a8727e393c02a85253401155fedd5d9f39b8800d5d25533019444bb7b219d4e32615d19d9a96cf425f6ccf48d0d3944a26575f1d64d5d0b72c5a5f01f90b7f32ada28b70443dd2140a2a1bc903a1e1b16ffd5101f27240efb2ca5383c30d9622160f97800320647591f686a943d61dbdce4646170f3ea07abac534bfad68d255e6e6d9684e168de4fae399dfb2d07c89e185b8f884b8401d384408a5143b5eb1285d59cee6510a13aabf5df43255178fb672924978235009759cb21b84bda2dd7387e218902b6a9d075c4a7c5218194b4f4ac7da2f6bc0b8402e50813415f43ed535ebcc3401487dbc1fdf1de5e3ce9ed4d00b8d502fdd9ee317c2cc0975ed88a58932ceb9a25288983b00ce74f440c146e1477111a1a370910c8401020304c70a84040506070bc3808080";

    #[test]
    fn encodes_istanbul_extra_to_rlp() {
        for extra_bytes in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            prepend_vanity(ISTANBUL_EXTRA_DUMPED),
        ] {
            let decoded_ist = IstanbulExtra::from_rlp(&extra_bytes).unwrap();

            println!("decoded_ist: {:?}", decoded_ist);
            let vanity = IstanbulExtraVanity::from_bytes(&extra_bytes);
            let encoded_ist_bytes = decoded_ist.to_rlp(vanity.unwrap());

            assert_eq!(encoded_ist_bytes, extra_bytes);
        }
    }

    #[test]
    fn decodes_istanbul_extra_from_rlp() {
        let expected = vec![
            IstanbulExtra {
                added_validators: to_address_vec(vec![
                    "44add0ec310f115a0e603b2d7db9f067778eaf8a",
                    "294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212",
                ]),
                added_public_keys: vec![],
                added_g1_public_keys: vec![],
                removed_validators: Integer::from(12),
                seal: Vec::new(),
                aggregated_seal: IstanbulAggregatedSeal::new(),
                parent_aggregated_seal: IstanbulAggregatedSeal::new(),
            },
        ];

        for (bytes, expected_ist) in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
        ]
        .iter()
        .zip(expected)
        {
            let parsed = IstanbulExtra::from_rlp(&bytes).unwrap();

            assert_eq!(parsed, expected_ist);
        }
    }

    #[test]
    fn rejects_insufficient_vanity() {
        let bytes = vec![0; ISTANBUL_EXTRA_VANITY_LENGTH - 1];

        assert!(IstanbulExtra::from_rlp(&bytes).is_err());
    }

    #[test]
    fn serializes_and_deserializes_to_json() {
        for bytes in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            prepend_vanity(&ISTANBUL_EXTRA_DUMPED),
        ]
        .iter()
        {
            let parsed = IstanbulExtra::from_rlp(&bytes).unwrap();
            let json_string = serde_json::to_string(&parsed).unwrap();
            let deserialized_from_json: IstanbulExtra = serde_json::from_str(&json_string).unwrap();

            assert_eq!(parsed, deserialized_from_json);
        }
    }

    fn prepend_vanity(data: &str) -> Vec<u8> {
        let data = hex::decode(data).unwrap();
        let vanity = IstanbulExtraVanity::default();

        [&vanity[..], &data[..]].concat()
    }

    fn to_address_vec(addresses: Vec<&str>) -> Vec<Address> {
        addresses
            .iter()
            .map(|address| {
                Address::from_bytes(hex::decode(address).unwrap().as_slice())
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    #[test]
    fn validates_epoch_math() {
        assert_eq!(
            vec![
                get_epoch_number(0, 3),
                get_epoch_number(3, 3),
                get_epoch_number(4, 3)
            ],
            vec![0, 1, 2]
        );
    }

    #[test]
    fn gen_test_data() {
        let mut header = Header {
            parent_hash: to_hash(
                "7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7",
            ),
            coinbase: to_hash("908D0FDaEAEFbb209BDcb540C2891e75616154b3"),
            root: to_hash("ecc60e00b3fe5ce9f6e1a10e5469764daf51f1fe93c22ec3f9a7583a80357217"),
            tx_hash: to_hash("d35d334d87c0cc0a202e3756bf81fae08b1575f286c7ee7a3f8df4f0f3afc55d"),
            receipt_hash: to_hash(
                "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            ),
            bloom: Bloom::default(),
            number: Integer::from(1000),
            gas_limit: Default::default(),
            gas_used: Integer::from(0x5208),
            time: Integer::from(0x5c47775c),
            extra: Vec::default(),
            min_digest: [0; HASH_LENGTH],
            nonce: [0; NONCE_LENGTH],
            base_fee: Default::default(),
        };


        let extra = IstanbulExtra {
            added_validators: to_address_vec(vec![
            ]),
            added_public_keys: vec![],
            added_g1_public_keys: vec![],
            removed_validators: Integer::from(0),
            seal: Vec::new(),
            aggregated_seal: IstanbulAggregatedSeal::new(),
            parent_aggregated_seal: IstanbulAggregatedSeal::new(),
        };

        header.extra = extra.to_rlp(&IstanbulExtraVanity::default());
        let hash = header.hash_without_seal().unwrap();




        println!("{}", serde_json::to_string(&header).unwrap())
    }

    pub fn to_hash<T>(data: &str) -> T
        where
            T: FromBytes + Clone,
    {
        T::from_bytes(&hex::decode(data).unwrap())
            .unwrap()
            .to_owned()
    }
}
