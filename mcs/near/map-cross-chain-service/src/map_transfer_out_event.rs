use crate::prover::{Address, MapEvent, EthEventParams};
use ethabi::{ParamType, Token};
use hex::ToHex;
use near_sdk::{AccountId, Balance, CryptoHash};
use near_sdk::serde::{Serialize, Deserialize};
use map_light_client::proof::LogEntry;

/// Data that was emitted by the Ethereum Locked event.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct MapTransferOutEvent {
    #[serde(with = "crate::bytes::hexstring")]
    pub map_bridge_address: Address,
    pub from_chain: u128,
    pub to_chain: u128,
    pub from: String,
    pub to: AccountId,
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
            ("token".to_string(), ParamType::String, true),
            ("from".to_string(), ParamType::String, true),
            ("orderId".to_string(), ParamType::FixedBytes(32), true),
            ("fromChain".to_string(), ParamType::Uint(256), false),
            ("toChain".to_string(), ParamType::Uint(256), false),
            ("to".to_string(), ParamType::String, false),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("toChainToken".to_string(), ParamType::String, false),
        ]
    }

    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &LogEntry) -> Option<Self> {
        let event = MapEvent::from_log_entry_data("mapTransferOut", MapTransferOutEvent::event_params(), data)?;
        let token = event.log.params[0].value.clone().to_string().unwrap();
        let from = event.log.params[1].value.clone().to_string().unwrap();
        let order_id: CryptoHash = event.log.params[2].value.clone().to_fixed_bytes().unwrap().try_into()
            .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length 32 but it was {}", v.len()));
        let from_chain = event.log.params[3].value.clone().to_uint().unwrap().as_u128();
        let to_chain = event.log.params[4].value.clone().to_uint().unwrap().as_u128();
        let to = event.log.params[5].value.clone().to_string().unwrap();
        let amount = event.log.params[6].value.clone().to_uint().unwrap().as_u128();
        let to_chain_token = event.log.params[7].value.clone().to_string().unwrap();
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferOutEvent {
    pub token: String,
    pub from: String,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub from_chain: u128,
    pub to_chain: u128,
    pub to: AccountId,
    pub amount: Balance,
    pub to_chain_token: String,
}

impl std::fmt::Display for TransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositOutEvent {
    pub token: String,
    pub from: String,
    pub to: String,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub amount: u128,
}

impl std::fmt::Display for DepositOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(result, event_data);
    }
}
