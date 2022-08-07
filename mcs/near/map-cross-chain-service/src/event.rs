use crate::prover::{Address, MapEvent, EthEventParams};
use ethabi::{Bytes, ParamType, Token};
use near_sdk::{Balance, CryptoHash};
use near_sdk::serde::{Serialize, Deserialize};
use map_light_client::proof::LogEntry;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use rlp::{Encodable, RlpStream};

/// Data that was emitted by the Ethereum Locked event.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct MapTransferOutEvent {
    #[serde(with = "crate::bytes::hexstring")]
    pub map_bridge_address: Address,
    pub from_chain: u128,
    pub to_chain: u128,
    pub from: Bytes,
    pub to: Bytes,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: Bytes,
    pub to_chain_token: Bytes,
    pub amount: Balance,
}
/*
event mapTransferOut(address indexed token, address indexed from, bytes32 indexed orderId,
uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
 */
impl MapTransferOutEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("token".to_string(), ParamType::Bytes, true),
            ("from".to_string(), ParamType::Bytes, true),
            ("orderId".to_string(), ParamType::FixedBytes(32), true),
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
            vec![
                self.token.clone(),
                self.from.clone(),
                self.order_id.clone().to_vec(),
            ],
            vec![
                Token::Uint(self.from_chain.into()),
                Token::Uint(self.to_chain.into()),
                Token::Bytes(self.to.clone()),
                Token::Uint(self.amount.into()),
                Token::Bytes(self.to_chain_token.clone()),
            ],
        )
    }
}

// impl std::fmt::Display for MapTransferOutEvent {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "token: {}; from: {};  orderId: {}; fromChain: {}; toChain: {}; to: {}; amount: {}; toChainToken: {}",
//             self.token, self.from, hex::encode(self.order_id), self.from_chain, self.to_chain, self.to, self.amount, self.to_chain_token
//         )
//     }
// }

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferOutEvent {
    pub token: String,
    pub from: String,
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

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositOutEvent {
    pub token: String,
    pub from: String,
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
                "address": "0x1e2b8b93443cc0a3ac97b7844092fa6950f47435",
                "topics": [
                    "0x1d7c4ab437b83807c25950ac63192692227b29e3205a809db6a4c3841836eb02",
                    "0x000000000000000000000000ec3e016916ba9f10762e33e03e8556409d096fb4",
                    "0x000000000000000000000000ec3e016916ba9f10762e33e03e8556409d096fb4",
                    "0x7240c481582afe87a6cb9f590fa966ad7ded67e66365a9b98bde1d198a99bec1"
                ],
                "data": "0x00000000000000000000000000000000000000000000000000000000000000d4000000000000000000000000000000000000000000000000000000004e45415300000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000f70616e646172722e746573746e65740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b6d63735f746f6b656e5f30000000000000000000000000000000000000000000"
            }
        ]"#;

        let logs: Vec<LogEntry> = serde_json::from_str(&logs_str).unwrap();
        assert_eq!(1, logs.len(), "should have only 1 log");

       let event = MapTransferOutEvent::from_log_entry_data(logs.get(0).unwrap()).unwrap();
        assert_eq!("1e2b8b93443cc0a3ac97b7844092fa6950f47435", hex::encode(event.map_bridge_address.clone()));
        assert_eq!("ec3e016916ba9f10762e33e03e8556409d096fb4", hex::encode(event.token.clone()));
        assert_eq!("ec3e016916ba9f10762e33e03e8556409d096fb4", hex::encode(event.from.clone()));
        assert_eq!(212, event.from_chain);
        assert_eq!("pandarr.testnet", String::from_utf8(event.to.clone()).unwrap());
        assert_eq!(1313161555, event.to_chain);
        assert_eq!("mcs_token_0", String::from_utf8(event.to_chain_token.clone()).unwrap());
        assert_eq!(100, event.amount);

        let data = event.to_log_entry_data();
        let result = MapTransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result, event);
    }

    #[test]
    fn test_event_serialize() {
        let exp = "f87fa836623137353437346538393039346334346461393862393534656564656163343935323731643066a830303030353437346538393039346334346461393862393534656564656163343935323731643066a000000000000000000000000000000000000000000000000000000000000000006482c350833132338203e880";
        let event_data = TransferOutEvent {
            token: "6b175474e89094c44da98b954eedeac495271d0f".to_string(),
            from: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
            to: "123".try_to_vec().unwrap(),
            amount: 1000,
            from_chain: 100,
            to_chain: 50000,
            to_chain_token: "".to_string(),
            order_id: [0; 32],
        };

        let rlp = rlp::encode(&event_data);
        assert_eq!(exp, hex::encode(&rlp))
    }

    #[test]
    fn test_log() {
        let logs_str = r#"[
        {
            "address": "0xe2123fa0c94db1e5baeff348c0e7aecd15a11b45",
            "topics": [
            "0x1d7c4ab437b83807c25950ac63192692227b29e3205a809db6a4c3841836eb02",
            "0x000000000000000000000000078f684c7d3bf78bdbe8bef93e56998442dc8099",
            "0x000000000000000000000000078f684c7d3bf78bdbe8bef93e56998442dc8099",
            "0xb2c235986cf359714f68eb1bd939adefcfb0bb2447c2c6473e569945f897761f"
            ],
            "data": "0x00000000000000000000000000000000000000000000000000000000000071a0000000000000000000000000000000000000000000000000000000004e45415300000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000206feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd300000000000000000000000000000000000000000000000000000000000000206feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3"
        }
        ]"#;

        let logs: Vec<LogEntry> = serde_json::from_str(&logs_str).unwrap();

        let events: Vec<MapTransferOutEvent> = logs.iter()
            .map(|e| MapTransferOutEvent::from_log_entry_data(e))
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect();


        assert_eq!(1, events.len());

        let event = events.get(0).unwrap();

        assert_eq!("078F684c7d3bf78BDbe8bEf93E56998442dc8099".to_lowercase(), hex::encode(&event.token));
        assert_eq!("078F684c7d3bf78BDbe8bEf93E56998442dc8099".to_lowercase(), hex::encode(&event.from));
        assert_eq!("b2c235986cf359714f68eb1bd939adefcfb0bb2447c2c6473e569945f897761f".to_string(), hex::encode(event.order_id));
        assert_eq!(29088, event.from_chain);
        assert_eq!(1313161555, event.to_chain);
        assert_eq!("6feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3".to_lowercase(), String::from_utf8(event.to.clone()).unwrap());
        assert_eq!(100, event.amount);
        assert_eq!("6feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3".to_lowercase(), String::from_utf8(event.to_chain_token.clone()).unwrap());
        assert_eq!("e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_lowercase(), hex::encode(event.map_bridge_address));
    }
}
