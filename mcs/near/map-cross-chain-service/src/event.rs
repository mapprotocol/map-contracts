use crate::prover::{Address, MapEvent, EthEventParams};
use ethabi::{ParamType, Token};
use near_sdk::{Balance, CryptoHash};
use near_sdk::serde::{Serialize, Deserialize};
use map_light_client::proof::LogEntry;
use rlp::{Encodable, RlpStream};

/// Data that was emitted by the Ethereum Locked event.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct MapTransferOutEvent {
    #[serde(with = "crate::bytes::hexstring")]
    pub map_bridge_address: Address,
    pub from_chain: u128,
    pub to_chain: u128,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: Vec<u8>,
    pub to_chain_token: Vec<u8>,
    pub amount: Balance,
}
/*
event mapTransferOut(address indexed token, address indexed from, bytes32 indexed orderId,
uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
 */
impl MapTransferOutEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("token".to_string(), ParamType::Bytes, false),
            ("from".to_string(), ParamType::Bytes, false),
            ("orderId".to_string(), ParamType::FixedBytes(32), false),
            ("fromChain".to_string(), ParamType::Uint(256), false),
            ("toChain".to_string(), ParamType::Uint(256), false),
            ("to".to_string(), ParamType::Bytes, false),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("toChainToken".to_string(), ParamType::Bytes, false),
        ]
    }

    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &LogEntry) -> Option<Self> {
        let event = MapEvent::from_log_entry_data("mapTransferOut", MapTransferOutEvent::event_params(), data)?;
        let token = event.log.params[0].value.clone().to_bytes().unwrap();
        let from = event.log.params[1].value.clone().to_bytes().unwrap();
        let order_id: CryptoHash = event.log.params[2].value.clone().to_fixed_bytes().unwrap().try_into()
            .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length 32 but it was {}", v.len()));
        let from_chain = event.log.params[3].value.clone().to_uint().unwrap().as_u128();
        let to_chain = event.log.params[4].value.clone().to_uint().unwrap().as_u128();
        let to = event.log.params[5].value.clone().to_bytes().unwrap();
        let amount = event.log.params[6].value.clone().to_uint().unwrap().as_u128();
        let to_chain_token = event.log.params[7].value.clone().to_bytes().unwrap();
        Some(Self {
            map_bridge_address: event.mcs_address,
            token,
            from,
            to,
            order_id,
            amount,
            from_chain,
            to_chain,
            to_chain_token,
        })
    }

    pub fn to_log_entry_data(&self) -> LogEntry {
        MapEvent::to_log_entry_data(
            "mapTransferOut",
            MapTransferOutEvent::event_params(),
            self.map_bridge_address,
            vec![],
            vec![
                Token::Bytes(self.token.clone()),
                Token::Bytes(self.from.clone()),
                Token::FixedBytes(self.order_id.clone().to_vec()),
                Token::Uint(self.from_chain.into()),
                Token::Uint(self.to_chain.into()),
                Token::Bytes(self.to.clone()),
                Token::Uint(self.amount.into()),
                Token::Bytes(self.to_chain_token.clone()),
            ],
        )
    }
}

impl std::fmt::Display for MapTransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "token: {:?}; from: {:?};  orderId: {}; fromChain: {}; toChain: {}; to: {}; amount: {}; toChainToken: {}",
            self.token, self.from, hex::encode(self.order_id), self.from_chain, self.to_chain,
            String::from_utf8(self.to.clone()).unwrap(), self.amount, String::from_utf8(self.to_chain_token.clone()).unwrap()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferOutEvent {
    pub token: String,
    pub from: String,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub from_chain: u128,
    pub to_chain: u128,
    pub to: Vec<u8>,
    pub amount: Balance,
    pub to_chain_token: String,
}

impl Encodable for TransferOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(8);

        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.order_id.as_ref());
        s.append(&self.from_chain);
        s.append(&self.to_chain);
        s.append(&self.to);
        s.append(&self.amount);
        s.append(&self.to_chain_token);
    }
}

impl std::fmt::Display for TransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositOutEvent {
    pub token: String,
    pub from: String,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub to: Vec<u8>,
    pub amount: u128,
}

impl Encodable for DepositOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5);

        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.order_id.as_ref());
        s.append(&self.to);
        s.append(&self.amount);
    }
}


impl std::fmt::Display for DepositOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_event_data() {
        let logs_str = r#"[
            {
                "address": "0xe2123fa0c94db1e5baeff348c0e7aecd15a11b45",
                "topics": [
                    "0xaca0a1067548270e80c1209ec69b5381d80bdaf345ad70cf7f00af9c6ed3f9b4"
                ],
                "data": "0x00000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140d1edb03b4a0fe5d7b378a7beddd84a81ff8d6f2cf15607be44ba3518b541818f00000000000000000000000000000000000000000000000000000000000000d4000000000000000000000000000000000000000000000000000000004e4541530000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f70616e646172722e746573746e65740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b6d63735f746f6b656e5f30000000000000000000000000000000000000000000"
            }
        ]"#;

        let logs: Vec<LogEntry> = serde_json::from_str(&logs_str).unwrap();
        assert_eq!(1, logs.len(), "should have only 1 log");

        let mut event = MapTransferOutEvent::from_log_entry_data(logs.get(0).unwrap()).unwrap();
        assert_eq!("ec3e016916ba9f10762e33e03e8556409d096fb4", hex::encode(event.token.clone()));
        assert_eq!("ec3e016916ba9f10762e33e03e8556409d096fb4", hex::encode(event.from.clone()));
        assert_eq!(212, event.from_chain);
        assert_eq!(1313161555, event.to_chain);
        assert_eq!("pandarr.testnet", String::from_utf8(event.to.clone()).unwrap());
        assert_eq!(100, event.amount);
        assert_eq!("mcs_token_0", String::from_utf8(event.to_chain_token.clone()).unwrap());
        assert_eq!("e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_lowercase(), hex::encode(event.map_bridge_address));


        let data = event.to_log_entry_data();
        let result = MapTransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result, event);
    }
}
