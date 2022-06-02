use crate::types::errors::{ Kind};
use crate::serialization::rlp::{
    big_int_to_rlp_compat_bytes, rlp_field_from_bytes, rlp_to_big_int,
};
use crate::slice_as_array_ref;
use crate::traits::{DefaultFrom, FromBytes};
use crate::types::header::Address;
use num_bigint::BigInt as Integer;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};

/// PUBLIC_KEY_LENGTH represents the number of bytes used to represent BLS public key
pub const PUBLIC_KEY_LENGTH: usize = 64;

/// SerializedPublicKey is a public key of a validator that is used to i.e sign the validator set
/// in the header
pub type SerializedPublicKey = [u8; PUBLIC_KEY_LENGTH];

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

    /// The public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_public_keys: Vec<SerializedPublicKey>,

    /// The BLS public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_g1_public_keys: Vec<SerializedPublicKey>,

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
        s.begin_list(6);
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

        let added_g1_public_keys: Result<Vec<SerializedPublicKey>, DecoderError> = rlp
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

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Num;

    // tiny example to assert validity of basic data
    const ISTANBUL_EXTRA_TINY: &str = "f6ea9444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212c00c80c3808080c3808080";

    // real life example (0x2a300 block from Celo main network) to assert validty of all the fields
    const ISTANBUL_EXTRA_DUMPED: &str = "d983010000846765746889676f312e31332e3130856c696e7578000000000000f90c6ef9020d94e8cea87569eb67b24196a0d3e2dfdd52f64d037d94e3b49907192aaabe84512629ce7c01be816bbb9d940439ad8b999acc4cda10b2aacae058732c4f2f8694edba91c8cc9fd2d88c5fa18aa1ffd6fac6959281947f4afdae66b590a90f2250e2d78bd27b4294a5dd94dcf109c09081042e1bfef3f21fa93ba61d3a3d5a944e986af5c4796432bfa2a3fb7ade1b0e9cda988b94f6b0e762344a8d10f77dd4f8f918e7d1c83be2f3946fabedc952b6bc7a11548f412e61f3af7ebe0e8394f2435e0b468e4c45eb4875127470da5c6842909d9424cee8c02fe517e19eaa783e0b7eadea66e5e6c294a063831defdc73d5a8a3f8993fdf4f108d5b044b94b5011538fceec8cbda40af59be09c552aead71389409b353c4e4c4d836b4a4ca4686370b78bc7a215294764a4ee67687994ababfe97f3a81411209011158949acda9211cfc11ed40fe05e20bb1429118199c9194e3feea837446183a233317f1c5d804b98fd50c6b945f897eed6797c0e7dfce3d97c48bf7ed032ee32194fe91ff8733bd21e3277190c0a00c300a9431d7879410ea19a286daf51dfdafbd3656a1262be4980a58946632c91b891e26229c86c59340917591be732028943edbd914a59a79417d0274fad7d80e39f8b219f794440a4917c9c833aa3984322f5c9e852cf300936594ff3bd8067551f79ac3ebbb1fa00d114e4b53b642947c079b1af73dfad29b89e4787aa4ce82475fe66ff90992b8608d483d8a391fce6ed73e9d4d2169d0bd569f3cedd6923350cd7195bab999517cb002564ef28d8c0205ca4ce3d42f7701fba58217bbd83288c7897904af6c2bedbfbbf220bc2a0239dfe505b07628b16f61e091a313cc7c34c7ee624b849e3f80b860a91c5c4939664eec9a8182ef7da28610450385ab8c74cc8f25c68c87ff1141bc9c7bd1259d674e21cb7ffb48b4034e017b529c80ec314eebcf1d360690c5777dcff42995327d5b1a9858cacc35076661704e0b50bd1502d7f8ebf18bdc4ca501b8603386f70c4fd8b3d9773af643ec70a6ae6d90365941170c44d6145b9725b89adf1489dadf0ca3d133019a82eb434cf200c2504bb8ce0282294c586644d29be422957a508fc6692ef711a1438b7d266598466ca6f8e5ac0e037df84885c6f92b00b860334b3d31ac71cdf8ad37ec78c314878f7d9b976b7b0e9cae348aba3c13a8d78fb1ce4be8c4a93e48a68467d14b503000b6f9dac56dc638e513e76b76e7e9787c31ccd2be57edba5c1ee9687703af14ea651ff497e1bd3541b0b99506d62eba80b8602c4fa91541e13318ba5bb9c5bf2204e320d333dcfd61bd6207476e2610ed2fc0133e919bfa6150563b66a015d6347a01947fe549bdc9bc0adf8c36fe42d47c1f25d67a74196eba4fc1a02b65cb9c0bd05860e33b059d86a119955327b4495500b86043c11c8346d8393ac9f0d5035f5c85ef69155e25ba9d78dba46b5fb4edfb7e05db29f2ffb8d0db07f03d4b704f3c4c010cf0f889cdd868ef6691f1acec197b7dcf8e8ac545913768ae7e1a97aefd4d450094b0bedcc8db78a649c90c1e9c6281b860af795e211ad3ed4e46e11b94dadba56f201dc3a36eb41f6ad6fbeae119088293a54bab374aecd2305037c330e59230018065c0869480b6ab0b490e8a6f766d0b5930220c0df4c7c898e50d87bba477ce850b03e7a00b936a487e27c3f674a981b8607ef1b4aa6216a489a587e96df4ad109ce1279240176664a0845b837f83d09206ce20adab624fa618ab0916091bc23a00eb6a400c521734ee4e67dc895912f7f6f13cc9c6f1298dae95b7aa9f752481097d4b27b9fd09a5eb70958aaae3eade00b8604eac7f432065192a70dd7cd724d9ce3a466ca987087bf75c361bf132d7611352093d3430029d6021695a3dd717cfa3007d1110c281c40da6b9ba287b2865cf8082b59e67242b0aa3d4576586047b926b9dd3bd383b9a1651d7776759956c1d00b8605effa652455d663ad1f7d02b76cea6333551f7ed4c4c0bab0d0078bc34ae24e81426c81b157b57cb6078b7804d925d0012d3fd8c8e0be5f2521db02cd0c6523ec7b2d43e91f69eb3433c0bf249cda1258d859e8463ca2b6c1b19bfc4ea018901b8606e92d50b8b2cbb1d03a6d92b8b0a53685f38225771854c596a21baaca797b2a36bf0c783bad5cad1093c4441470aed00a4077fb8fe382e201a038afb0a32448a982263ad75405f127baf5935e35ec6e1cec624e4ae619fd92945664ac407f680b860f6914b8a9c6227e00eb732298034585b1519e51bdd3cea087d59ccd78c9fe6ea37c626a970a18924c2e4d3dfa2f37900994be34deca5a4fab594990251b4ae85da8845ede08b3190ace9f7653fcffec4b599bd1ba76dee9a7e3bdcdd79990d80b86050f45c41b3e8398b7f04734b8461e1fc137dd211c8a3bd657dbde0f773563a582cf74b037c7a4879787a1695242cde00bd366d131bc43d71efb41dd959c74b0b51469bd409d1bff7f3abb400dc8309d880f1551e18a1000af7a7f3a52af42580b860068f6a5e22ec195f52015004199aa8fdda8735bc6b7eb2d030f4807b01293131ee36f0cf78d45e6bbfbacc05d931700006c7c8d7fad54137d10d9d09abd33f92114e6f5d12eccc69bd2adbac1cc78061c97fa9d5b30fa5636402039766a6e980b8609261e0674d685b4e29e12fd55aac90128577523c82979572cbe1453bf2c638ecba7d3cfb67708d9427a365e5e2c384000edaa42a23b59cb723f77330fd5b6d2ef3a93d518b32e0eeb195a2fbb99ef431001445df09c8e88b0dc783a94bb48f01b8608ea9ae193ad42ddbcc7222858bbc3ec5f342e9a00fde212b244c6ceedb07827cfb4b50e8f3365e80d35fa64a412e24001bc4628ae2e34df40f29348b32762be6a2da3e65a9199e9ae5285d048e7caaa0cfdf3aaaac0cec0499688d0c2b7eab80b860a37b611c1ac3453817699183e3e703cd79452080919ed2e7ecaf9a14ee6602cedb44346bd7920d7fde6bd50762bc6700014aa1db8445d898708a090baa113e39e63b0541583bb4d4ab3811631e0ac903a817269e42ab042de79f7605e9b7c700b86085e7fad58fbdb49110b38142d90753f107560b6a52eb2674d223fdd7c97f9da36e9de4d16deb3eac169807082bc89c004a52ed567d9065f4e5b6dba9e279fc0f46d976da4e562d5873da8fdc8a8fb52d647a1394e46e998938c7e2a2b22a1080b860139d447bc50301235f68e91e31f799e9194aa98f1c61295d5fe207e47e6b4c903e4c66e7efd3993039d633633d6fc200cf17a5b49a3b90db55e10679edb6876c6bb3aa138c5bdd073c411287847ae858d2e7b0cf53b80d056568be0fa5074000b8602e8f2c8609cc8ef00c4f70d7e5aafe6908e8827606f559ab0268ec9b6725b88c7e6cfcc8cdc7061ef5cb4e8526a3090032b29d15ff64845f7451c299ea2d92fbc3d72961a16c9beea0c6ded4a0d2162550fe63752a64b884703d209b7d7d1e81b860ce50e9605cac46755090ba8f29e155834eefbe7d5fc0821ba00d7e6b1ff80df74bd04f989a319aaff8217d1f07d5a600a42f94bbd737aa4562f1e6a0f31c68029cb2924d3ca4e78cfe0dab82f785ba6f09be625514f04957d457dc4f789d3001b860fbca8830e0e95a7b6d23121c279a125085f8c2bc39e40912689cbb2462c7cc1c6b8cfb7cd2fda28f5cbe8aa5d2ee8b01a20659e6a55d7143db9b889ffd6bc96f4cdcde900597c957f56fd0e0c943fe13a9e3ac376cac6d8f48db7a8082292501b860edb8634cb66357c8ec2534c82dc4f3dfc04cd0c2a10468015ad7303d61868dcdf5f6f2aa96822cf4f9abce05e98e49000ac9cedd9d89f7d41e263d375969fdb5ff9babc3cc761778c088f17b2064e0ceed08c61934e550d3e673267c85f83281b8602d9da563d6b418ecaa931a669603e6736d7012ddd2c44ab429ce88123942e9f91f5c7c2c87d07d2d5af905fff2e8c2006e7bee1e0522ad6d7eb8e36e897120592f2ce30a77ab2d58ff131374cd11d8b6f5b1f7fa1aa4379e9e9607f1a9228380b860fc55fe153f5f7ab3914bc2c36f71055224fc42ac8bba492a6b901d0cd7e16e95f07726d5b137fd9778b2390278b057003b7c115b6d88085c11f612a8a47d5255d18f078db4033c1eb0de07366d1548587a1f6361ccb2e14f524403815c335580890138100001801e0005b8419c0095d64903827be6b1ca1072109074d30aae4e6a209bf2c3c4c83bc38c1a29551a08d975a5aad0a64be5b85dd5a4fb9bc50a6220668e23382cd362d9672ba900f83c8901eca063eebfbbfdf9b0428e302ff6aab449d68fbdde248a4494b0db5f166a0a64244defcaea0e8342f8d1361bfe60df5e9180087fb703f57b8180f83c8901efeffffffffffffbb03669d77a600391712293fac898ff03637cf87789ff7b4ba7479554f8acbfd864f9d454246b4788024d95c5063e039c8080";

    #[test]
    fn encodes_istanbul_extra_to_rlp() {
        for extra_bytes in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            hex::decode(&ISTANBUL_EXTRA_DUMPED).unwrap(),
        ] {
            let decoded_ist = IstanbulExtra::from_rlp(&extra_bytes).unwrap();
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
            IstanbulExtra {
                added_validators: to_address_vec(vec![
                    "e8cea87569eb67b24196a0d3e2dfdd52f64d037d",
                    "e3b49907192aaabe84512629ce7c01be816bbb9d",
                    "0439ad8b999acc4cda10b2aacae058732c4f2f86",
                    "edba91c8cc9fd2d88c5fa18aa1ffd6fac6959281",
                    "7f4afdae66b590a90f2250e2d78bd27b4294a5dd",
                    "dcf109c09081042e1bfef3f21fa93ba61d3a3d5a",
                    "4e986af5c4796432bfa2a3fb7ade1b0e9cda988b",
                    "f6b0e762344a8d10f77dd4f8f918e7d1c83be2f3",
                    "6fabedc952b6bc7a11548f412e61f3af7ebe0e83",
                    "f2435e0b468e4c45eb4875127470da5c6842909d",
                    "24cee8c02fe517e19eaa783e0b7eadea66e5e6c2",
                    "a063831defdc73d5a8a3f8993fdf4f108d5b044b",
                    "b5011538fceec8cbda40af59be09c552aead7138",
                    "09b353c4e4c4d836b4a4ca4686370b78bc7a2152",
                    "764a4ee67687994ababfe97f3a81411209011158",
                    "9acda9211cfc11ed40fe05e20bb1429118199c91",
                    "e3feea837446183a233317f1c5d804b98fd50c6b",
                    "5f897eed6797c0e7dfce3d97c48bf7ed032ee321",
                    "fe91ff8733bd21e3277190c0a00c300a9431d787",
                    "10ea19a286daf51dfdafbd3656a1262be4980a58",
                    "6632c91b891e26229c86c59340917591be732028",
                    "3edbd914a59a79417d0274fad7d80e39f8b219f7",
                    "440a4917c9c833aa3984322f5c9e852cf3009365",
                    "ff3bd8067551f79ac3ebbb1fa00d114e4b53b642",
                    "7c079b1af73dfad29b89e4787aa4ce82475fe66f",
                ]),
                added_public_keys: to_serialized_pub_key_vec(vec![
                    "8d483d8a391fce6ed73e9d4d2169d0bd569f3cedd6923350cd7195bab999517cb002564ef28d8c0205ca4ce3d42f7701fba58217bbd83288c7897904af6c2bedbfbbf220bc2a0239dfe505b07628b16f61e091a313cc7c34c7ee624b849e3f80",
                    "a91c5c4939664eec9a8182ef7da28610450385ab8c74cc8f25c68c87ff1141bc9c7bd1259d674e21cb7ffb48b4034e017b529c80ec314eebcf1d360690c5777dcff42995327d5b1a9858cacc35076661704e0b50bd1502d7f8ebf18bdc4ca501",
                    "3386f70c4fd8b3d9773af643ec70a6ae6d90365941170c44d6145b9725b89adf1489dadf0ca3d133019a82eb434cf200c2504bb8ce0282294c586644d29be422957a508fc6692ef711a1438b7d266598466ca6f8e5ac0e037df84885c6f92b00",
                    "334b3d31ac71cdf8ad37ec78c314878f7d9b976b7b0e9cae348aba3c13a8d78fb1ce4be8c4a93e48a68467d14b503000b6f9dac56dc638e513e76b76e7e9787c31ccd2be57edba5c1ee9687703af14ea651ff497e1bd3541b0b99506d62eba80",
                    "2c4fa91541e13318ba5bb9c5bf2204e320d333dcfd61bd6207476e2610ed2fc0133e919bfa6150563b66a015d6347a01947fe549bdc9bc0adf8c36fe42d47c1f25d67a74196eba4fc1a02b65cb9c0bd05860e33b059d86a119955327b4495500",
                    "43c11c8346d8393ac9f0d5035f5c85ef69155e25ba9d78dba46b5fb4edfb7e05db29f2ffb8d0db07f03d4b704f3c4c010cf0f889cdd868ef6691f1acec197b7dcf8e8ac545913768ae7e1a97aefd4d450094b0bedcc8db78a649c90c1e9c6281",
                    "af795e211ad3ed4e46e11b94dadba56f201dc3a36eb41f6ad6fbeae119088293a54bab374aecd2305037c330e59230018065c0869480b6ab0b490e8a6f766d0b5930220c0df4c7c898e50d87bba477ce850b03e7a00b936a487e27c3f674a981",
                    "7ef1b4aa6216a489a587e96df4ad109ce1279240176664a0845b837f83d09206ce20adab624fa618ab0916091bc23a00eb6a400c521734ee4e67dc895912f7f6f13cc9c6f1298dae95b7aa9f752481097d4b27b9fd09a5eb70958aaae3eade00",
                    "4eac7f432065192a70dd7cd724d9ce3a466ca987087bf75c361bf132d7611352093d3430029d6021695a3dd717cfa3007d1110c281c40da6b9ba287b2865cf8082b59e67242b0aa3d4576586047b926b9dd3bd383b9a1651d7776759956c1d00",
                    "5effa652455d663ad1f7d02b76cea6333551f7ed4c4c0bab0d0078bc34ae24e81426c81b157b57cb6078b7804d925d0012d3fd8c8e0be5f2521db02cd0c6523ec7b2d43e91f69eb3433c0bf249cda1258d859e8463ca2b6c1b19bfc4ea018901",
                    "6e92d50b8b2cbb1d03a6d92b8b0a53685f38225771854c596a21baaca797b2a36bf0c783bad5cad1093c4441470aed00a4077fb8fe382e201a038afb0a32448a982263ad75405f127baf5935e35ec6e1cec624e4ae619fd92945664ac407f680",
                    "f6914b8a9c6227e00eb732298034585b1519e51bdd3cea087d59ccd78c9fe6ea37c626a970a18924c2e4d3dfa2f37900994be34deca5a4fab594990251b4ae85da8845ede08b3190ace9f7653fcffec4b599bd1ba76dee9a7e3bdcdd79990d80",
                    "50f45c41b3e8398b7f04734b8461e1fc137dd211c8a3bd657dbde0f773563a582cf74b037c7a4879787a1695242cde00bd366d131bc43d71efb41dd959c74b0b51469bd409d1bff7f3abb400dc8309d880f1551e18a1000af7a7f3a52af42580",
                    "068f6a5e22ec195f52015004199aa8fdda8735bc6b7eb2d030f4807b01293131ee36f0cf78d45e6bbfbacc05d931700006c7c8d7fad54137d10d9d09abd33f92114e6f5d12eccc69bd2adbac1cc78061c97fa9d5b30fa5636402039766a6e980",
                    "9261e0674d685b4e29e12fd55aac90128577523c82979572cbe1453bf2c638ecba7d3cfb67708d9427a365e5e2c384000edaa42a23b59cb723f77330fd5b6d2ef3a93d518b32e0eeb195a2fbb99ef431001445df09c8e88b0dc783a94bb48f01",
                    "8ea9ae193ad42ddbcc7222858bbc3ec5f342e9a00fde212b244c6ceedb07827cfb4b50e8f3365e80d35fa64a412e24001bc4628ae2e34df40f29348b32762be6a2da3e65a9199e9ae5285d048e7caaa0cfdf3aaaac0cec0499688d0c2b7eab80",
                    "a37b611c1ac3453817699183e3e703cd79452080919ed2e7ecaf9a14ee6602cedb44346bd7920d7fde6bd50762bc6700014aa1db8445d898708a090baa113e39e63b0541583bb4d4ab3811631e0ac903a817269e42ab042de79f7605e9b7c700",
                    "85e7fad58fbdb49110b38142d90753f107560b6a52eb2674d223fdd7c97f9da36e9de4d16deb3eac169807082bc89c004a52ed567d9065f4e5b6dba9e279fc0f46d976da4e562d5873da8fdc8a8fb52d647a1394e46e998938c7e2a2b22a1080",
                    "139d447bc50301235f68e91e31f799e9194aa98f1c61295d5fe207e47e6b4c903e4c66e7efd3993039d633633d6fc200cf17a5b49a3b90db55e10679edb6876c6bb3aa138c5bdd073c411287847ae858d2e7b0cf53b80d056568be0fa5074000",
                    "2e8f2c8609cc8ef00c4f70d7e5aafe6908e8827606f559ab0268ec9b6725b88c7e6cfcc8cdc7061ef5cb4e8526a3090032b29d15ff64845f7451c299ea2d92fbc3d72961a16c9beea0c6ded4a0d2162550fe63752a64b884703d209b7d7d1e81",
                    "ce50e9605cac46755090ba8f29e155834eefbe7d5fc0821ba00d7e6b1ff80df74bd04f989a319aaff8217d1f07d5a600a42f94bbd737aa4562f1e6a0f31c68029cb2924d3ca4e78cfe0dab82f785ba6f09be625514f04957d457dc4f789d3001",
                    "fbca8830e0e95a7b6d23121c279a125085f8c2bc39e40912689cbb2462c7cc1c6b8cfb7cd2fda28f5cbe8aa5d2ee8b01a20659e6a55d7143db9b889ffd6bc96f4cdcde900597c957f56fd0e0c943fe13a9e3ac376cac6d8f48db7a8082292501",
                    "edb8634cb66357c8ec2534c82dc4f3dfc04cd0c2a10468015ad7303d61868dcdf5f6f2aa96822cf4f9abce05e98e49000ac9cedd9d89f7d41e263d375969fdb5ff9babc3cc761778c088f17b2064e0ceed08c61934e550d3e673267c85f83281",
                    "2d9da563d6b418ecaa931a669603e6736d7012ddd2c44ab429ce88123942e9f91f5c7c2c87d07d2d5af905fff2e8c2006e7bee1e0522ad6d7eb8e36e897120592f2ce30a77ab2d58ff131374cd11d8b6f5b1f7fa1aa4379e9e9607f1a9228380",
                    "fc55fe153f5f7ab3914bc2c36f71055224fc42ac8bba492a6b901d0cd7e16e95f07726d5b137fd9778b2390278b057003b7c115b6d88085c11f612a8a47d5255d18f078db4033c1eb0de07366d1548587a1f6361ccb2e14f524403815c335580"
                ]),
                removed_validators: Integer::from_str_radix("22486472945905303557", 10).unwrap(),
                seal: hex::decode("9c0095d64903827be6b1ca1072109074d30aae4e6a209bf2c3c4c83bc38c1a29551a08d975a5aad0a64be5b85dd5a4fb9bc50a6220668e23382cd362d9672ba900").unwrap(),
                aggregated_seal: IstanbulAggregatedSeal{
                    bitmap: Integer::from_str_radix("35497482140004384249", 10).unwrap(),
                    signature: hex::decode("428e302ff6aab449d68fbdde248a4494b0db5f166a0a64244defcaea0e8342f8d1361bfe60df5e9180087fb703f57b81").unwrap(),
                    round: Integer::from(0),
                },
                parent_aggregated_seal: IstanbulAggregatedSeal{
                    bitmap: Integer::from_str_radix("35736063043184885755", 10).unwrap(),
                    signature: hex::decode("3669d77a600391712293fac898ff03637cf87789ff7b4ba7479554f8acbfd864f9d454246b4788024d95c5063e039c80").unwrap(),
                    round: Integer::from(0),
                },
                added_g1_public_keys: vec![]
            },
        ];

        for (bytes, expected_ist) in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            hex::decode(&ISTANBUL_EXTRA_DUMPED).unwrap(),
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

    // #[test]
    // fn serializes_and_deserializes_to_json() {
    //     for bytes in vec![
    //         prepend_vanity(ISTANBUL_EXTRA_TINY),
    //         hex::decode(&ISTANBUL_EXTRA_DUMPED).unwrap(),
    //     ]
    //     .iter()
    //     {
    //         let parsed = IstanbulExtra::from_rlp(&bytes).unwrap();
    //         let json_string = serde_json::to_string(&parsed).unwrap();
    //         let deserialized_from_json: IstanbulExtra = serde_json::from_str(&json_string).unwrap();
    //
    //         assert_eq!(parsed, deserialized_from_json);
    //     }
    // }

    fn prepend_vanity(data: &str) -> Vec<u8> {
        let data = hex::decode(data).unwrap();
        let vanity = IstanbulExtraVanity::default();

        [&vanity[..], &data[..]].concat()
    }

    fn to_serialized_pub_key_vec(keys: Vec<&str>) -> Vec<SerializedPublicKey> {
        keys.iter()
            .map(|key| {
                SerializedPublicKey::from_bytes(hex::decode(key).unwrap().as_slice())
                    .unwrap()
                    .to_owned()
            })
            .collect()
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
}
