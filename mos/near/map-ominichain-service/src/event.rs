use crate::prover::{Address, EVMEvent, EthEventParams};
use crate::traits::Transferable;
use crate::SwapAction;
use ethabi::{ParamType, Token};
use map_light_client::proof::LogEntry;
use near_sdk::env::panic_str;
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, AccountId, CryptoHash};
use rlp::{Encodable, RlpStream};

const PATH_SEPARATOR: &str = "X";

const TRANSFER_OUT_TYPE: &str = "2ef1cdf83614a69568ed2c96a275dd7fb2e63a464aa3a0ffe79f55d538c8b3b5";
const DEPOSIT_OUT_TYPE: &str = "150bd848adaf4e3e699dcac82d75f111c078ce893375373593cc1b9208998377";
const SWAP_OUT_TYPE: &str = "ca1cf8cebf88499429cca8f87cbca15ab8dafd06702259a5344ddce89ef3f3a5";

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum MCSEvent {
    Transfer(TransferOutEvent),
    Deposit(DepositOutEvent),
    Swap(SwapOutEvent),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferInParam {
    pub order_id: CryptoHash,
    pub to_chain_token: AccountId,
    pub to: AccountId,
    pub amount: U128,
}

impl MCSEvent {
    pub fn emit(&self) {
        match self {
            MCSEvent::Transfer(event) => event.emit(),
            MCSEvent::Deposit(event) => event.emit(),
            MCSEvent::Swap(event) => event.emit(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum MsgType {
    Transfer,
    Deposit,
    Swap,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferOutEvent {
    pub from_chain: U128,
    pub to_chain: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: Vec<u8>,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub amount: U128,
    pub to_chain_token: Vec<u8>,
}

impl TransferOutEvent {
    /*
    event mapTransferOut(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId, bytes token, bytes from, bytes to, uint256 amount, bytes toChainToken);
     */
    fn event_params() -> EthEventParams {
        vec![
            ("fromChain".to_string(), ParamType::Uint(256), true),
            ("toChain".to_string(), ParamType::Uint(256), true),
            ("orderId".to_string(), ParamType::FixedBytes(32), false),
            ("token".to_string(), ParamType::Bytes, false),
            ("from".to_string(), ParamType::Bytes, false),
            ("to".to_string(), ParamType::Bytes, false),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("toChainToken".to_string(), ParamType::Bytes, false),
        ]
    }
    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &LogEntry) -> Option<(Address, Self)> {
        let event = EVMEvent::from_log_entry_data("mapTransferOut", Self::event_params(), data)?;
        let from_chain = event.log.params[0]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let to_chain = event.log.params[1]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let order_id: CryptoHash = event.log.params[2]
            .value
            .clone()
            .to_fixed_bytes()?
            .try_into()
            .ok()?;
        let token = event.log.params[3].value.clone().to_bytes()?;
        let from = event.log.params[4].value.clone().to_bytes()?;
        let to = event.log.params[5].value.clone().to_bytes()?;
        let amount = event.log.params[6]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let to_chain_token = event.log.params[7].value.clone().to_bytes()?;
        Some((
            event.address,
            Self {
                from_chain,
                to_chain,
                order_id,
                token,
                from,
                to,
                amount,
                to_chain_token,
            },
        ))
    }

    pub fn emit(&self) {
        log!("transfer out: {}", serde_json::to_string(self).unwrap());
        log!("{}{}", TRANSFER_OUT_TYPE, self);
    }

    pub fn get_to_chain_token(&self) -> AccountId {
        String::from_utf8(self.to_chain_token.clone())
            .unwrap()
            .parse()
            .unwrap()
    }

    pub fn get_to_account(&self) -> AccountId {
        String::from_utf8(self.to.clone()).unwrap().parse().unwrap()
    }
}

impl Encodable for TransferOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(8);

        s.append(&self.from_chain.0);
        s.append(&self.to_chain.0);
        s.append(&self.order_id.as_ref());
        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.amount.0);
        s.append(&self.to_chain_token);
    }
}

impl std::fmt::Display for TransferOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}

impl Transferable for TransferOutEvent {
    fn get_transfer_in_param(&self) -> TransferInParam {
        TransferInParam {
            order_id: self.order_id,
            to_chain_token: String::from_utf8(self.to_chain_token.clone())
                .unwrap()
                .parse()
                .unwrap(),
            to: String::from_utf8(self.to.clone()).unwrap().parse().unwrap(),
            amount: self.amount,
        }
    }

    fn basic_check(&self) {
        assert!(
            env::is_valid_account_id(self.to.as_slice()),
            "invalid to address: {:?}",
            self.to
        );
        assert!(
            env::is_valid_account_id(self.to_chain_token.as_slice()),
            "invalid to chain token address: {:?}",
            self.to_chain_token
        );
        assert!(self.amount.gt(&U128(0)), "amount should be greater than 0");
    }

    fn get_to_chain(&self) -> U128 {
        self.to_chain
    }

    fn get_order_id(&self) -> CryptoHash {
        self.order_id
    }

    fn get_transfer_in_token(&self) -> AccountId {
        String::from_utf8(self.to_chain_token.clone())
            .unwrap()
            .parse()
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositOutEvent {
    pub from_chain: U128,
    pub to_chain: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: String,
    pub from: String,
    pub to: Vec<u8>,
    pub amount: U128,
}

impl DepositOutEvent {
    pub fn emit(&self) {
        log!("deposit out: {}", serde_json::to_string(self).unwrap());
        log!("{}{}", DEPOSIT_OUT_TYPE, self);
    }
}

impl Encodable for DepositOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7);

        s.append(&self.from_chain.0);
        s.append(&self.to_chain.0);
        s.append(&self.order_id.as_ref());
        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.amount.0);
    }
}

impl std::fmt::Display for DepositOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapParam {
    pub amount_in: U128,
    pub min_amount_out: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub path: Vec<u8>,
    pub router_index: U64,
}

impl SwapParam {
    pub fn to_swap_action(&self) -> SwapAction {
        let path = String::from_utf8(self.path.clone()).unwrap();
        let split: Vec<&str> = path.split(PATH_SEPARATOR).collect();

        SwapAction {
            pool_id: self.router_index.into(),
            token_in: split.first().unwrap().parse().unwrap(),
            amount_in: if self.amount_in.0 == 0 {
                None
            } else {
                Some(self.amount_in)
            },
            token_out: split.get(1).unwrap().parse().unwrap(),
            min_amount_out: self.min_amount_out,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapData {
    pub swap_param: Vec<SwapParam>,
    #[serde(with = "crate::bytes::hexstring")]
    pub target_token: Vec<u8>,
    #[serde(with = "crate::bytes::hexstring")]
    pub map_target_token: Address,
}

impl SwapData {
    pub fn abi_decode(data: Vec<u8>) -> Option<Self> {
        let swap_param_types = ParamType::Tuple(vec![
            Box::new(ParamType::Uint(256)),
            Box::new(ParamType::Uint(256)),
            Box::new(ParamType::Bytes),
            Box::new(ParamType::Uint(64)),
        ]);

        let param_type = vec![
            ParamType::Array(Box::new(swap_param_types)),
            ParamType::Bytes,
            ParamType::Address,
        ];

        let tokens = ethabi::decode(param_type.as_slice(), data.as_slice()).ok()?;
        if tokens.len() != 3 {
            return None;
        }
        let swap_param_tokens = tokens[0].clone().to_array()?;
        let mut swap_param: Vec<SwapParam> = Vec::new();
        for swap_param_token in swap_param_tokens {
            if let Token::Tuple(inner_tokens) = swap_param_token {
                if inner_tokens.len() != 4 {
                    return None;
                }
                swap_param.push(SwapParam {
                    amount_in: U128(inner_tokens[0].clone().to_uint()?.as_u128()),
                    min_amount_out: U128(inner_tokens[1].clone().to_uint()?.as_u128()),
                    path: inner_tokens[2].clone().to_bytes()?,
                    router_index: U64(inner_tokens[3].clone().to_uint()?.as_u64()),
                })
            } else {
                return None;
            }
        }
        let target_token = tokens[1].clone().to_bytes()?;
        let map_target_token = tokens[2].clone().to_address()?.0;
        Some(Self {
            swap_param,
            target_token,
            map_target_token,
        })
    }

    pub fn abi_encode(&self) -> Vec<u8> {
        let mut swap_param_token: Vec<Token> = Vec::new();
        for param in self.swap_param.clone() {
            let tuple = vec![
                Token::Uint(param.amount_in.0.into()),
                Token::Uint(param.min_amount_out.0.into()),
                Token::Bytes(param.path),
                Token::Uint(param.router_index.0.into()),
            ];
            swap_param_token.push(Token::Tuple(tuple));
        }
        let tokens: Vec<Token> = vec![
            Token::Array(swap_param_token),
            Token::Bytes(self.target_token.clone()),
            Token::Address(self.map_target_token.into()),
        ];

        ethabi::encode(tokens.as_slice())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapOutEvent {
    pub from_chain: U128,
    pub to_chain: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: Vec<u8>,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub amount: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub swap_data: Vec<u8>,

    pub raw_swap_data: SwapData,
    pub src_token: String,
    pub src_amount: U128,
    #[serde(with = "crate::bytes::hexstring")]
    pub dst_token: Vec<u8>,
}

impl SwapOutEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("fromChain".to_string(), ParamType::Uint(256), true),
            ("toChain".to_string(), ParamType::Uint(256), true),
            ("orderId".to_string(), ParamType::FixedBytes(32), false),
            ("token".to_string(), ParamType::Bytes, false),
            ("from".to_string(), ParamType::Bytes, false),
            ("to".to_string(), ParamType::Bytes, false),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("swapData".to_string(), ParamType::Bytes, false),
        ]
    }

    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &LogEntry) -> Option<(Address, Self)> {
        let event = EVMEvent::from_log_entry_data("mapSwapOut", Self::event_params(), data)?;
        let from_chain = event.log.params[0]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let to_chain = event.log.params[1]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let order_id: CryptoHash = event.log.params[2]
            .value
            .clone()
            .to_fixed_bytes()?
            .try_into()
            .ok()?;
        let token = event.log.params[3].value.clone().to_bytes()?;
        let from = event.log.params[4].value.clone().to_bytes()?;
        let to = event.log.params[5].value.clone().to_bytes()?;
        let amount = event.log.params[6]
            .value
            .clone()
            .to_uint()?
            .as_u128()
            .into();
        let swap_data = event.log.params[7].value.clone().to_bytes()?;
        let raw_swap_data = SwapData::abi_decode(swap_data.clone())?;
        Some((
            event.address,
            Self {
                from_chain,
                to_chain,
                order_id,
                token,
                from,
                to,
                amount,
                swap_data,
                raw_swap_data,
                src_token: "".to_string(),
                src_amount: U128(0),
                dst_token: vec![],
            },
        ))
    }

    pub fn emit(&self) {
        log!("swap out: {}", serde_json::to_string(self).unwrap());
        log!("{}{}", SWAP_OUT_TYPE, self);
    }

    pub fn to_transfer_out_event(&self) -> Option<TransferOutEvent> {
        if self.raw_swap_data.swap_param.is_empty() {
            Some(TransferOutEvent {
                from_chain: self.from_chain,
                to_chain: self.to_chain,
                order_id: self.order_id,
                token: self.token.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
                amount: self.amount,
                to_chain_token: self.raw_swap_data.target_token.clone(),
            })
        } else {
            None
        }
    }

    pub fn get_token_in(&self) -> AccountId {
        if self.raw_swap_data.swap_param.is_empty() {
            String::from_utf8(self.raw_swap_data.target_token.clone())
                .unwrap()
                .parse()
                .unwrap()
        } else {
            let path =
                String::from_utf8(self.raw_swap_data.swap_param.get(0).unwrap().path.clone())
                    .unwrap();
            let split: Vec<&str> = path.split(PATH_SEPARATOR).collect();
            split.first().unwrap().parse().unwrap()
        }
    }

    pub fn get_token_out(&self) -> AccountId {
        String::from_utf8(self.raw_swap_data.target_token.clone())
            .unwrap()
            .parse()
            .unwrap()
    }

    pub fn get_to_account(&self) -> AccountId {
        String::from_utf8(self.to.clone()).unwrap().parse().unwrap()
    }
}

impl Encodable for SwapOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(8);

        s.append(&self.from_chain.0);
        s.append(&self.to_chain.0);
        s.append(&self.order_id.as_ref());
        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.amount.0);
        s.append(&self.swap_data);
    }
}

impl std::fmt::Display for SwapOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}

impl Transferable for SwapOutEvent {
    fn get_transfer_in_param(&self) -> TransferInParam {
        TransferInParam {
            order_id: self.order_id,
            to_chain_token: if self.raw_swap_data.swap_param.is_empty() {
                String::from_utf8(self.raw_swap_data.target_token.clone())
                    .unwrap()
                    .parse()
                    .unwrap()
            } else {
                let path =
                    String::from_utf8(self.raw_swap_data.swap_param.get(0).unwrap().path.clone())
                        .unwrap();
                let split: Vec<&str> = path.split(PATH_SEPARATOR).collect();
                split.first().unwrap().parse().unwrap()
            },
            to: if self.raw_swap_data.swap_param.is_empty() {
                String::from_utf8(self.to.clone()).unwrap().parse().unwrap()
            } else {
                env::current_account_id()
            },
            amount: self.amount,
        }
    }

    fn basic_check(&self) {
        assert!(
            env::is_valid_account_id(self.to.as_slice()),
            "invalid to address: {:?}",
            self.to
        );
        assert!(self.amount.gt(&U128(0)), "amount should be greater than 0");

        assert!(
            env::is_valid_account_id(self.raw_swap_data.target_token.as_slice()),
            "invalid target token address: {:?}",
            self.raw_swap_data.target_token
        );

        if !self.raw_swap_data.swap_param.is_empty() {
            for (i, param) in self.raw_swap_data.swap_param.iter().enumerate() {
                assert_eq!(0, param.amount_in.0, "amount in should be == 0");
                assert!(
                    param.min_amount_out.0 > 0,
                    "min amount out should be greater than 0"
                );
                let path = String::from_utf8(param.path.clone())
                    .unwrap_or_else(|_| panic_str("invalid swap param path"));
                let split: Vec<&str> = path.split(PATH_SEPARATOR).collect();
                assert_eq!(2, split.len(), "invalid path format: {}", path);

                assert!(
                    env::is_valid_account_id(split[0].as_bytes()),
                    "invalid account id in path: {:?}",
                    split[0]
                );
                assert!(
                    env::is_valid_account_id(split[1].as_bytes()),
                    "invalid account id in path: {:?}",
                    split[1]
                );

                if i == self.raw_swap_data.swap_param.len() - 1 {
                    assert_eq!(
                        split[1].as_bytes(),
                        self.raw_swap_data.target_token.as_slice(),
                        "target token should be equal to the last token out"
                    );
                }
            }
        }
    }

    fn get_to_chain(&self) -> U128 {
        self.to_chain
    }

    fn get_order_id(&self) -> CryptoHash {
        self.order_id
    }

    fn get_transfer_in_token(&self) -> AccountId {
        if self.raw_swap_data.swap_param.is_empty() {
            String::from_utf8(self.raw_swap_data.target_token.clone())
                .unwrap()
                .parse()
                .unwrap()
        } else {
            let path = String::from_utf8(self.raw_swap_data.swap_param[0].path.clone()).unwrap();
            let split: Vec<&str> = path.split(PATH_SEPARATOR).collect();
            split.first().unwrap().parse().unwrap()
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapInEvent {
    #[serde(with = "crate::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token_out: AccountId,
    pub amount_out: U128,
}

impl SwapInEvent {
    pub fn emit(&self) {
        log!("swap in: {}", serde_json::to_string(self).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_eth_address;
    use ethabi::Token;
    use hex;
    use near_sdk::AccountId;
    use std::str::FromStr;
    use std::string::String;
    use tiny_keccak::keccak256;

    impl TransferOutEvent {
        pub fn to_log_entry_data(&self, map_bridge_address: Address) -> LogEntry {
            EVMEvent::to_log_entry_data(
                "mapTransferOut",
                TransferOutEvent::event_params(),
                map_bridge_address,
                vec![
                    self.from_chain.0.clone().to_be_bytes().to_vec(),
                    self.to_chain.0.clone().to_be_bytes().to_vec(),
                ],
                vec![
                    Token::FixedBytes(self.order_id.clone().to_vec()),
                    Token::Bytes(self.token.clone()),
                    Token::Bytes(self.from.clone()),
                    Token::Bytes(self.to.clone()),
                    Token::Uint(self.amount.0.into()),
                    Token::Bytes(self.to_chain_token.clone()),
                ],
            )
        }
    }

    impl SwapOutEvent {
        pub fn to_log_entry_data(&self, map_bridge_address: Address) -> LogEntry {
            EVMEvent::to_log_entry_data(
                "mapSwapOut",
                SwapOutEvent::event_params(),
                map_bridge_address,
                vec![
                    self.from_chain.0.clone().to_be_bytes().to_vec(),
                    self.to_chain.0.clone().to_be_bytes().to_vec(),
                ],
                vec![
                    Token::FixedBytes(self.order_id.clone().to_vec()),
                    Token::Bytes(self.token.clone()),
                    Token::Bytes(self.from.clone()),
                    Token::Bytes(self.to.clone()),
                    Token::Uint(self.amount.0.into()),
                    Token::Bytes(self.swap_data.clone()),
                ],
            )
        }
    }

    #[test]
    fn test_decode_transfer_event_data() {
        let logs_str = r#"[
            {
                "address": "0xe2123fa0c94db1e5baeff348c0e7aecd15a11b45",
                "topics": [
                    "0x44ff77018688dad4b245e8ab97358ed57ed92269952ece7ffd321366ce078622",
                    "0x00000000000000000000000000000000000000000000000000000000000000d4",
                    "0x000000000000000000000000000000000000000000000000000000004e454153"
                ],
                "data": "0x64e604787cbf194841e7b68d7cd28786f6c9a0a3ab9f8b0a0e87cb4387ab010700000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000000000000000000000000000000000000000000000000000000000000000000014223e016916ba9f10762e33e03e8556409d096f22000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f70616e646172722e746573746e6574000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000196d63735f746f6b656e5f302e6d63732e746573742e6e65617200000000000000"
            }
        ]"#;

        let logs: Vec<LogEntry> = serde_json::from_str(&logs_str).unwrap();
        assert_eq!(1, logs.len(), "should have only 1 log");

        let (mcs, event) = TransferOutEvent::from_log_entry_data(logs.get(0).unwrap()).unwrap();
        assert_eq!(
            "ec3e016916ba9f10762e33e03e8556409d096fb4",
            hex::encode(event.token.clone())
        );
        assert_eq!(
            "223e016916ba9f10762e33e03e8556409d096f22",
            hex::encode(event.from.clone())
        );
        assert_eq!(212, event.from_chain.0);
        assert_eq!(1313161555, event.to_chain.0);
        assert_eq!(
            "pandarr.testnet",
            String::from_utf8(event.to.clone()).unwrap()
        );
        assert_eq!(100, event.amount.0);
        assert_eq!(
            "mcs_token_0.mcs.test.near",
            String::from_utf8(event.to_chain_token.clone()).unwrap()
        );
        assert_eq!(
            "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_lowercase(),
            hex::encode(mcs)
        );

        let data = event.to_log_entry_data(mcs);
        let result = TransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result.1, event);
    }

    #[test]
    fn test_encode_transfer_event() {
        let event = TransferOutEvent {
            from_chain: U128(212),
            to_chain: U128(1313161555),
            order_id: keccak256("123".as_bytes()),
            token: hex::decode("ec3e016916ba9f10762e33e03e8556409d096fb4").unwrap(),
            from: hex::decode("223e016916ba9f10762e33e03e8556409d096f22").unwrap(),
            to: "pandarr.test.near".as_bytes().to_vec(),
            amount: U128(100),
            to_chain_token: "wrap.test.near".as_bytes().to_vec(),
        };

        let data = event.to_log_entry_data([1; 20]);
        let result = TransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result.0, [1; 20]);
        assert_eq!(result.1, event);

        println!("{:?}", serde_json::to_string(&data).unwrap());
    }

    #[test]
    fn test_swap_event_data() {
        let mut swap_param: Vec<SwapParam> = Vec::new();
        // swap_param.push(SwapParam {
        //     amount_in: U128(0),
        //     min_amount_out: U128(1),
        //     path: "usdc.map007.testnetXusdt.map007.testnet"
        //         .as_bytes()
        //         .to_vec(),
        //     router_index: U64(1821),
        // });

        // let raw_swap_data = SwapData {
        //     swap_param,
        //     target_token: "usdt.map007.testnet".as_bytes().to_vec(),
        //     map_target_token: [4; 20],
        // };
        swap_param.push(SwapParam {
            amount_in: U128(0),
            min_amount_out: U128(1),
            path: "usdc.map007.testnetXwrap.testnet".as_bytes().to_vec(),
            router_index: U64(1786),
        });

        let raw_swap_data = SwapData {
            swap_param,
            target_token: "wrap.testnet".as_bytes().to_vec(),
            map_target_token: [4; 20],
        };
        let event = SwapOutEvent {
            from_chain: U128(212),
            to_chain: U128(1360100178526210),
            order_id: [8; 32],
            token: vec![1; 20],
            from: vec![2; 20],
            to: "pandarr.testnet".as_bytes().to_vec(),
            amount: U128(100000),
            swap_data: raw_swap_data.abi_encode(),
            raw_swap_data,
            src_token: "".to_string(),
            src_amount: U128(0),
            dst_token: vec![],
        };

        let mcs = validate_eth_address("630105189c7114667a7179Aa57f07647a5f42B7F".to_string());

        let data = event.to_log_entry_data(mcs);
        let result = SwapOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result.0, mcs);
        assert_eq!(result.1, event);

        println!("{}", serde_json::to_string(&data).unwrap());
        println!("{}", serde_json::to_string(&event.raw_swap_data).unwrap())
    }

    #[test]
    fn test_encode_swap_data() {
        let exp = "00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000280000000000000000000000000b6c1b689291532d11172fb4c204bf13169ec0dca0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b746f6b656e312e6d61703030372e746573746e657458746f6b656e322e6d61703030372e746573746e65740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002b746f6b656e322e6d61703030372e746573746e657458746f6b656e332e6d61703030372e746573746e6574000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c777261702e746573746e65740000000000000000000000000000000000000000";

        let mut swap_param: Vec<SwapParam> = Vec::new();
        swap_param.push(SwapParam {
            amount_in: U128(0),
            min_amount_out: U128(1),
            path: "token1.map007.testnetXtoken2.map007.testnet"
                .as_bytes()
                .to_vec(),
            router_index: U64(0),
        });
        swap_param.push(SwapParam {
            amount_in: U128(0),
            min_amount_out: U128(1),
            path: "token2.map007.testnetXtoken3.map007.testnet"
                .as_bytes()
                .to_vec(),
            router_index: U64(1),
        });

        let swap_data = SwapData {
            swap_param,
            target_token: "wrap.testnet".as_bytes().to_vec(),
            map_target_token: validate_eth_address(
                "B6c1b689291532D11172Fb4C204bf13169EC0dCA".to_string(),
            ),
        };

        let byte_str = hex::encode(swap_data.abi_encode().as_slice());
        assert_eq!(exp, byte_str);

        let swap_data_dec = SwapData::abi_decode(hex::decode(exp).unwrap()).unwrap();
        assert_eq!(swap_data, swap_data_dec)
    }
}
