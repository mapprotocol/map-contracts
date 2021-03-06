use crate::prover::{Address, MapEvent, EthEventParams};
use ethabi::{ParamType, Token};
use near_sdk::{AccountId, Balance, CryptoHash};
use near_sdk::serde::{Serialize, Deserialize};
use map_light_client::proof::LogEntry;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

/// Data that was emitted by the Ethereum Locked event.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct MapTransferOutEvent {
    #[serde(with = "crate::bytes::hexstring")]
    pub map_bridge_address: Address,
    pub from_chain: u128,
    pub to_chain: u128,
    pub from: String,
    pub to: String,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: String,
    pub to_chain_token: String,
    pub amount: Balance,
}
/*
event mapTransferOut(address indexed token, address indexed from, bytes32 indexed orderId,
uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
 */
impl MapTransferOutEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("token".to_string(), ParamType::Address, true),
            ("from".to_string(), ParamType::Address, true),
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
        let token = hex::encode(event.log.params[0].value.clone().to_address().unwrap().0);
        let from = hex::encode(event.log.params[1].value.clone().to_address().unwrap().0);
        let order_id: CryptoHash = event.log.params[2].value.clone().to_fixed_bytes().unwrap().try_into()
            .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length 32 but it was {}", v.len()));
        let from_chain = event.log.params[3].value.clone().to_uint().unwrap().as_u128();
        let to_chain = event.log.params[4].value.clone().to_uint().unwrap().as_u128();
        let to = hex::encode(event.log.params[5].value.clone().to_bytes().unwrap());
        let amount = event.log.params[6].value.clone().to_uint().unwrap().as_u128();
        let to_chain_token = hex::encode(event.log.params[7].value.clone().to_bytes().unwrap());
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
                self.token.clone().into_bytes(),
                self.from.clone().into_bytes(),
                self.order_id.clone().to_vec(),
            ],
            vec![
                Token::Uint(self.from_chain.into()),
                Token::Uint(self.to_chain.into()),
                Token::String(self.to.clone()),
                Token::Uint(self.amount.into()),
                Token::String(self.to_chain_token.clone()),
            ],
        )
    }
}

impl std::fmt::Display for MapTransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "token: {}; from: {};  orderId: {}; fromChain: {}; toChain: {}; to: {}; amount: {}; toChainToken: {}",
            self.token, self.from, hex::encode(self.order_id), self.from_chain, self.to_chain, self.to, self.amount, self.to_chain_token
        )
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
// #[serde(crate = "near_sdk::serde")]
pub struct TransferOutEvent {
    pub token: String,
    pub from: String,
    // #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub from_chain: u128,
    pub to_chain: u128,
    pub to: String,
    pub amount: Balance,
    pub to_chain_token: String,
}

impl std::fmt::Display for TransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(borsh::to_vec(self).unwrap()))
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct DepositOutEvent {
    pub token: String,
    pub from: String,
    pub to: String,
    pub order_id: CryptoHash,
    pub amount: u128,
}

impl std::fmt::Display for DepositOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.try_to_vec().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_event_data() {
        let event_data = MapTransferOutEvent {
            map_bridge_address: [0u8; 20],
            token: "6b175474e89094c44da98b954eedeac495271d0f".to_string(),
            from: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
            to: "123".to_string(),
            amount: 1000,
            from_chain: 0,
            to_chain: 0,
            to_chain_token: "".to_string(),
            order_id: [1; 32],
        };
        let data = event_data.to_log_entry_data();
        let result = MapTransferOutEvent::from_log_entry_data(&data);
        // assert_eq!(result, event_data);
    }

    #[test]
    fn test_event_serialize() {
        let event_data = TransferOutEvent {
            token: "6b175474e89094c44da98b954eedeac495271d0f".to_string(),
            from: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
            to: "123".to_string(),
            amount: 1000,
            from_chain: 0,
            to_chain: 0,
            to_chain_token: "".to_string(),
            order_id: [1; 32],
        };
        // assert_eq!(result, event_data);
        let data = "1f0000006d63735f746f6b656e5f302e6d63732e70616e646172722e746573746e65740f00000070616e646172722e746573746e6574445c15cde59e54edb751d2c91f3a241e87f7eaad794d78d867f5a85ec37cc0cf5341454e000000000000000000000000010000000000000000000000000000002a0000003078373630376339636464373333643863646130613634343833396563326261633566613138306564346400000000000000000000000000000000000000";
        let event = hex::decode(data).unwrap();
        let result:TransferOutEvent = TransferOutEvent::try_from_slice(&event).unwrap();
        println!("{}, {}, {}, {}, {}, {}, {}, {}",
                 result.token, result.from, result.to, result.amount, result.from_chain, result.to_chain, result.to_chain_token, hex::encode(&result.order_id));
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

        assert_eq!("078F684c7d3bf78BDbe8bEf93E56998442dc8099".to_lowercase(), event.token);
        assert_eq!("078F684c7d3bf78BDbe8bEf93E56998442dc8099".to_lowercase(), event.from);
        assert_eq!("b2c235986cf359714f68eb1bd939adefcfb0bb2447c2c6473e569945f897761f".to_string(), hex::encode(event.order_id));
        assert_eq!(29088, event.from_chain);
        assert_eq!(1313161555, event.to_chain);
        assert_eq!("6feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3".to_lowercase(), event.to);
        assert_eq!(100, event.amount);
        assert_eq!("6feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3".to_lowercase(), event.to_chain_token);
        assert_eq!("e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_lowercase(), hex::encode(event.map_bridge_address));
    }
}
