use std::collections::HashSet;
use admin_controlled::{AdminControlled, Mask};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U128};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PublicKey, PromiseOrValue, PromiseResult, CryptoHash, log};

type EthereumAddress = [u8; 20];

pub use map_transfer_out_event::MapTransferOutEvent;
use prover::*;
pub use prover::validate_eth_address;
use std::convert::TryInto;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use map_light_client::proof::{LogEntry, ReceiptProof};
use crate::map_transfer_out_event::{TransferOutEvent, DepositOutEvent};

mod map_transfer_out_event;
pub mod prover;
mod bytes;

near_sdk::setup_alloc!();

const MCS_TOKEN_BINARY: &'static [u8] = include_bytes!("../../mcs-token/target/wasm32-unknown-unknown/release/mcs_token.wasm");

const MAP_CHAIN_ID: u128 = 22776;

const NO_DEPOSIT: Balance = 0;

/// Initial balance for the BridgeToken contract to cover storage and related.
const BRIDGE_TOKEN_INIT_BALANCE: Balance = 3_000_000_000_000_000_000_000_000; // 3e24yN, 3N

/// Gas to initialize BridgeToken contract.
const BRIDGE_TOKEN_NEW: Gas = 10_000_000_000_000;

/// Gas to call ft_transfer on ext fungible contract
const FT_TRANSFER_GAS: Gas = 80_000_000_000_000;

/// Gas to call mint method on bridge token.
const MINT_GAS: Gas = 10_000_000_000_000;

/// Gas to call mint method on bridge token.
const BURN_GAS: Gas = 10_000_000_000_000;

/// Gas to call ft_transfer_call when the target of deposit is a contract
const FT_TRANSFER_CALL_GAS: Gas = 80_000_000_000_000;

/// Gas to call near_withdraw when the to chain token address is empty
const NEAR_WITHDRAW_GAS: Gas = 80_000_000_000_000;

/// Gas to call near_deposit when the to chain token address is empty
const NEAR_DEPOSIT_GAS: Gas = 80_000_000_000_000;

/// Gas to call finish deposit method.
/// This doesn't cover the gas required for calling mint method.
const FINISH_DEPOSIT_GAS: Gas = 30_000_000_000_000;

/// Gas to call finish transfer out method.
const FINISH_TRANSFER_OUT_GAS: Gas = 30_000_000_000_000;

/// Gas to call finish update_metadata method.
const FINISH_UPDATE_METADATA_GAS: Gas = 5_000_000_000_000;

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = 50_000_000_000_000;

/// Amount of gas used by set_metadata in the factory, without taking into account
/// the gas consumed by the promise.
const OUTER_SET_METADATA_GAS: Gas = 15_000_000_000_000;

/// Amount of gas used by bridge token to set the metadata.
const SET_METADATA_GAS: Gas = 5_000_000_000_000;

/// Controller storage key.
const CONTROLLER_STORAGE_KEY: &[u8] = b"aCONTROLLER";

/// Metadata connector address storage key.
const METADATA_CONNECTOR_ETH_ADDRESS_STORAGE_KEY: &[u8] = b"aM_CONNECTOR";

/// Prefix used to store a map between tokens and timestamp `t`, where `t` stands for the
/// block on Ethereum where the metadata for given token was emitted.
/// The prefix is made specially short since it becomes more expensive with larger prefixes.
const TOKEN_TIMESTAMP_MAP_PREFIX: &[u8] = b"aTT";

#[derive(Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum ResultType {
    Withdraw {
        amount: Balance,
        token: EthereumAddress,
        recipient: EthereumAddress,
    },
    Lock {
        token: String,
        amount: Balance,
        recipient: EthereumAddress,
    },
}

const PAUSE_DEPLOY_TOKEN: Mask = 1 << 0;
const PAUSE_TRANSFER_IN: Mask = 1 << 1;
const PAUSE_TRANSFER_OUT_TOKEN: Mask = 1 << 2;
const PAUSE_TRANSFER_OUT_NATIVE: Mask = 1 << 3;
const PAUSE_DEPOSIT_OUT_TOKEN: Mask = 1 << 4;
const PAUSE_DEPOSIT_OUT_NATIVE: Mask = 1 << 5;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MapCrossChainService {
    /// The account of the map light client that we can use to prove
    pub map_client_account: AccountId,
    /// Address of the MAP bridge contract.
    pub map_bridge_address: Address,
    /// Set of created MCSToken contracts.
    pub mcs_tokens: UnorderedMap<String, HashSet<u128>>,
    /// Set of other fungible token contracts.
    pub fungible_tokens: UnorderedMap<String, HashSet<u128>>,
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<CryptoHash>,
    /// Public key of the account deploying the MCS contract.
    pub owner_pk: PublicKey,
    /// Balance required to register a new account in the MCSToken
    pub mcs_storage_transfer_in_required: Balance,
    // Wrap token for near
    pub wrapped_token: String,
    // wrap.near for mainnet
    // Near chain id
    // FIXME: get from env?
    pub near_chain_id: u128,
    // Nonce to generate order id
    pub nonce: u128,
    /// Mask determining all paused functions
    paused: Mask,
}

#[ext_contract(ext_self)]
pub trait ExtMapCrossChainService {
    fn finish_transfer_in(
        &self,
        events: Vec<MapTransferOutEvent>,
    ) -> Promise;

    fn finish_transfer_out(
        &self,
        event: TransferOutEvent,
    );
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_wnear_token)]
pub trait ExtWNearToken {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}

#[ext_contract(ext_mcs_token)]
pub trait ExtMCSToken {
    fn mint(&self, account_id: AccountId, amount: U128);
    fn burn(&self, account_id: AccountId, amount: U128);

    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn set_metadata(
        &mut self,
        name: Option<String>,
        symbol: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: Option<u8>,
        icon: Option<String>,
    );
}

pub fn assert_self() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}

struct Recipient {
    target: AccountId,
    message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
struct FungibleTokenMsg {
    typ: u8, // 0: Transfer or 1: Deposit
    to: String,
    to_chain: u128, // if typ is 1, it is omitted
}


/// `recipient` is the target account id receiving current ERC-20 tokens.
///
/// If `recipient` doesn't contain a semicolon (:) then it is interpreted as a NEAR account id
/// and token are minted as NEP-141 directly on `recipient` account id.
///
/// Otherwise, the format expected is: <target_address>:<message>
///
/// @target_address: Account id of the contract to transfer current funds
/// @message: Free form message to be send to the target using ft_transfer_call
///
/// The final message sent to the `target_address` has the format:
///
/// <message>
///
/// Where `message` is the free form string that was passed.
fn parse_recipient(recipient: String) -> Recipient {
    if recipient.contains(':') {
        let mut iter = recipient.split(':');
        let target = iter.next().unwrap().into();
        let message = iter.collect::<Vec<&str>>().join(":");

        Recipient {
            target,
            message: Some(message),
        }
    } else {
        Recipient {
            target: recipient,
            message: None,
        }
    }
}

#[near_bindgen]
impl MapCrossChainService {
    /// Initializes the contract.
    /// `prover_account`: NEAR account of the Near Prover contract;
    /// `locker_address`: Ethereum address of the locker contract, in hex.
    #[init]
    pub fn new(map_light_client: AccountId, map_bridge_address: String, wrapped_token: String, near_chain_id: u128) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            map_client_account: map_light_client,
            map_bridge_address: validate_eth_address(map_bridge_address),
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            used_events: UnorderedSet::new(b"u".to_vec()),
            owner_pk: env::signer_account_pk(),
            mcs_storage_transfer_in_required:
            near_contract_standards::fungible_token::FungibleToken::new(b"t".to_vec())
                .account_storage_usage as Balance
                * env::storage_byte_cost(),
            wrapped_token,
            near_chain_id,  // 1313161555 for testnet
            nonce: 0,
            paused: Mask::default(),
        }
    }

    /// Transfer from Map to NEAR based on the proof of the locked tokens or messages.
    /// Must attach enough NEAR funds to cover for storage of the proof.
    #[payable]
    pub fn transfer_in(&mut self, receipt_proof: ReceiptProof) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_IN);

        let events: Vec<MapTransferOutEvent> = receipt_proof.receipt.logs.iter()
            .map(|e| MapTransferOutEvent::from_log_entry_data(e))
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .filter(|e| e.map_bridge_address == self.map_bridge_address)
            .collect();
        assert_ne!(0, events.len(), "no cross chain event in the receipt");

        for event in events.iter() {
            assert!(
                self.mcs_tokens.get(&event.to_chain_token).is_some()
                    || self.fungible_tokens.get(&event.to_chain_token).is_some()
                    || event.to_chain_token == "",
                "to_chain_token {} is not mcs token or fungible token or empty",
                event.to_chain_token
            );
        }

        let event_len: u64 = events.len() as u64;

        ext_map_light_client::verify_proof_data(
            receipt_proof,
            &self.map_client_account,
            NO_DEPOSIT,
            VERIFY_LOG_ENTRY_GAS,
        )
            .then(ext_self::finish_transfer_in(
                events,
                &env::current_account_id(),
                env::attached_deposit(),
                (FINISH_DEPOSIT_GAS + MINT_GAS + FT_TRANSFER_CALL_GAS) * event_len,
            ))
    }

    #[payable]
    pub fn transfer_out_token(&mut self, token: String, to: String, amount: u128, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_IN);

        let from = env::signer_account_id();
        let order_id = self.get_order_id(&token, &from, &to, amount, to_chain);

        let event = TransferOutEvent {
            from_chain: self.near_chain_id,
            to_chain,
            from: from.clone(),
            to: to.clone(),
            order_id,
            token: token.clone(),
            to_chain_token: "".to_string(),
            amount,
        };
        if self.valid_mcs_token_out(&token, to_chain) {
            ext_mcs_token::burn(
                from,
                event.amount.into(),
                &event.token,
                self.mcs_storage_transfer_in_required,
                BURN_GAS,
            ).then(
                ext_self::finish_transfer_out(
                    event,
                    &env::current_account_id(),
                    env::attached_deposit() - self.mcs_storage_transfer_in_required,
                    FINISH_TRANSFER_OUT_GAS,
                )
            )
        } else if self.valid_fungible_token_out(&token, to_chain) {
            env::panic(format!("non mcs fungible token {} should called from fungible token directly", token).as_ref());
        } else {
            env::panic(format!("token {} to chain {} is not supported", token, to_chain).as_ref());
        }
    }

    #[payable]
    pub fn transfer_out_native(&mut self, to: String, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");

        let from = env::signer_account_id();

        let order_id = self.get_order_id(&"".to_string(), &from, &to, amount, to_chain);

        let event = TransferOutEvent{
            from_chain: self.near_chain_id,
            to_chain,
            from,
            to,
            order_id,
            token: "".to_string(),
            to_chain_token: "".to_string(),
            amount
        };

        ext_wnear_token::near_deposit(
            &self.wrapped_token,
            env::attached_deposit(),
            NEAR_DEPOSIT_GAS,
        ).then(
            ext_self::finish_transfer_out(
                event,
                &env::current_account_id(),
                0,
                FINISH_TRANSFER_OUT_GAS,
            )
        )
    }

    /// Finish transfer in once the proof was successfully validated. Can only be called by the contract
    /// itself.
    #[payable]
    pub fn finish_transfer_in(
        &mut self,
        events: Vec<MapTransferOutEvent>,
    ) -> Promise {
        assert_self();

        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get result from cross contract");

        let event = events.get(0).unwrap();
        let (mut promise, mut ret_deposit) = self.finish_transfer_in_single_event(event, env::attached_deposit());
        for (index, event) in events.iter().enumerate() {
            if index == 0 {
                continue;
            } else {
                let ret = self.finish_transfer_in_single_event(event, ret_deposit);
                promise = promise.then(ret.0);
                ret_deposit = ret.1;
            }
        }

        promise
    }

    fn finish_transfer_in_single_event(
        &mut self,
        event: &MapTransferOutEvent,
        cur_deposit: Balance,
    ) -> (Promise, Balance) {
        assert_self();

        assert_eq!(false, self.is_used_event(&event.order_id), "the event {} is used", event);

        let required_deposit = self.record_order_id(&event.order_id);

        assert!(cur_deposit >= required_deposit, "not enough deposit for record proof ");

        let Recipient { target, message } = parse_recipient(event.to.to_string());

        env::log(format!("Finish transfer in. Target:{} Message:{:?}", target, message).as_bytes());

        let mut ret_deposit = cur_deposit - required_deposit;

        if event.to_chain_token == "" {
            (ext_wnear_token::near_withdraw(
                event.amount.into(),
                &self.wrapped_token,
                1,
                NEAR_WITHDRAW_GAS,
            ).then(Promise::new(target).transfer(event.amount.into())), ret_deposit)
        } else if self.mcs_tokens.get(&event.to_chain_token).is_some() {
            assert!(ret_deposit >= self.mcs_storage_transfer_in_required, "not enough deposit for transfer in");

            ret_deposit = ret_deposit - self.mcs_storage_transfer_in_required;

            match message {
                Some(message) => (ext_mcs_token::mint(
                    env::current_account_id(),
                    event.amount.into(),
                    &event.to_chain_token,
                    self.mcs_storage_transfer_in_required,
                    MINT_GAS,
                )
                                      .then(ext_mcs_token::ft_transfer_call(
                                          target.try_into().unwrap(),
                                          event.amount.into(),
                                          None,
                                          message,
                                          &event.to_chain_token,
                                          1,
                                          FT_TRANSFER_CALL_GAS,
                                      )), ret_deposit),
                None => (ext_mcs_token::mint(
                    target,
                    event.amount.into(),
                    &event.to_chain_token,
                    self.mcs_storage_transfer_in_required,
                    MINT_GAS,
                ), ret_deposit),
            }
        } else {
            (ext_fungible_token::ft_transfer(
                target,
                event.amount.into(),
                None,
                &event.to_chain_token,
                1,
                FT_TRANSFER_GAS,
            ), ret_deposit)
        }
    }

    /// Finish transfer out once the nep141 token is burned from MCSToken contract or native token is transferred.
    pub fn finish_transfer_out(
        &mut self,
        event: TransferOutEvent,
    ) {
        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get result from cross contract");
        log!("{}", event);
    }

    #[payable]
    pub fn deposit_out_native(&mut self, from: String, to: String) {
        self.check_not_paused(PAUSE_DEPOSIT_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");

        let from = env::signer_account_id();

        let order_id = self.get_order_id(&"".to_string(), &from, &to, amount, MAP_CHAIN_ID);

        let event = DepositOutEvent{
            token: "".to_string(),
            from,
            to,
            order_id,
            amount
        };

        log!("{}", event);
    }

    #[payable]
    pub fn deploy_bridge_token(&mut self, address: String) -> Promise {
        self.check_not_paused(PAUSE_DEPLOY_TOKEN);
        let address = address.to_lowercase();
        // let _ = validate_eth_address(address.clone());
        assert!(
            self.mcs_tokens.get(&address).is_none(),
            "MCS token contract already exists."
        );
        let initial_storage = env::storage_usage() as u128;
        self.mcs_tokens.insert(&address, &Default::default());
        let current_storage = env::storage_usage() as u128;
        assert!(
            env::attached_deposit()
                >= BRIDGE_TOKEN_INIT_BALANCE
                + env::storage_byte_cost() * (current_storage - initial_storage),
            "Not enough attached deposit to complete bridge token creation"
        );

        let bridge_token_account_id = format!("{}.{}", address, env::current_account_id());
        Promise::new(bridge_token_account_id)
            .create_account()
            .transfer(BRIDGE_TOKEN_INIT_BALANCE)
            .add_full_access_key(self.owner_pk.clone())
            .deploy_contract(MCS_TOKEN_BINARY.to_vec())
            .function_call(
                b"new".to_vec(),
                b"{}".to_vec(),
                NO_DEPOSIT,
                BRIDGE_TOKEN_NEW,
            )
    }

    /// Checks whether the provided proof is already used
    pub fn is_used_event(&self, order_id: &CryptoHash) -> bool {
        self.used_events.contains(order_id)
    }

    /// Record order id to make sure it is not re-used later for anther deposit.
    fn record_order_id(&mut self, order_id: &CryptoHash) -> Balance {
        assert_self();
        let initial_storage = env::storage_usage();

        assert!(
            !self.used_events.contains(order_id),
            "Event cannot be reused for depositing."
        );
        self.used_events.insert(order_id);
        let current_storage = env::storage_usage();
        let required_deposit =
            Balance::from(current_storage - initial_storage) * env::storage_byte_cost();

        env::log(format!("RecordOrderId:{}", hex::encode(order_id)).as_bytes());
        required_deposit
    }

    /// Admin method to set metadata with admin/controller access
    pub fn set_metadata(
        &mut self,
        address: String,
        name: Option<String>,
        symbol: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: Option<u8>,
        icon: Option<String>,
    ) -> Promise {
        assert!(self.controller_or_self());
        ext_mcs_token::set_metadata(
            name,
            symbol,
            reference,
            reference_hash,
            decimals,
            icon,
            &address,
            env::attached_deposit(),
            env::prepaid_gas() - OUTER_SET_METADATA_GAS,
        )
    }

    /// Factory Controller. Controller has extra privileges inside this contract.
    pub fn controller(&self) -> Option<AccountId> {
        env::storage_read(CONTROLLER_STORAGE_KEY)
            .map(|value| String::from_utf8(value).expect("Invalid controller account id"))
    }

    pub fn set_controller(&mut self, controller: AccountId) {
        assert!(self.controller_or_self());
        assert!(env::is_valid_account_id(controller.as_bytes()));
        env::storage_write(CONTROLLER_STORAGE_KEY, controller.as_bytes());
    }

    pub fn controller_or_self(&self) -> bool {
        let caller = env::predecessor_account_id();
        caller == env::current_account_id()
            || self
            .controller()
            .map(|controller| controller == caller)
            .unwrap_or(false)
    }

    /// Return all registered tokens
    pub fn get_mcs_tokens(&self) -> Vec<String> {
        self.mcs_tokens.keys().collect::<Vec<_>>()
    }

    pub fn get_fungible_tokens(&self) -> Vec<String> {
        self.fungible_tokens.keys().collect::<Vec<_>>()
    }

    pub fn add_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.insert(to_chain);
    }

    pub fn remove_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain);
    }

    pub fn valid_mcs_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.mcs_tokens.get(&token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    pub fn valid_fungible_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.fungible_tokens.get(&token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    fn get_order_id(&mut self, token: &String, from: &AccountId, to: &String, amount: u128, to_chain_id: u128) -> CryptoHash {
        let mut data = self.nonce.try_to_vec().unwrap();
        data.extend(from.as_bytes());
        data.extend(to.as_bytes());
        data.extend(token.as_bytes());
        data.extend(amount.try_to_vec().unwrap());
        data.extend(self.near_chain_id.try_to_vec().unwrap());
        data.extend(to_chain_id.try_to_vec().unwrap());
        self.nonce = self.nonce + 1;
        CryptoHash::try_from(env::sha256(&data[..])).unwrap()
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for MapCrossChainService {
    fn ft_on_transfer(&mut self, sender_id: ValidAccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id();
        let transfer_msg: FungibleTokenMsg = serde_json::from_str(&msg).unwrap();
        let from = AccountId::from(sender_id);

        if transfer_msg.typ == 0 {
            assert!(self.valid_fungible_token_out(&token, transfer_msg.to_chain),
                    "transfer token {} to chain {} is not supported", token, transfer_msg.to_chain);

            let order_id = self.get_order_id(&token,
                                             &from,
                                             &transfer_msg.to,
                                             amount.0,
                                             transfer_msg.to_chain);
            let event = TransferOutEvent {
                from_chain: self.near_chain_id,
                to_chain: transfer_msg.to_chain,
                from,
                to: transfer_msg.to,
                order_id,
                token,
                to_chain_token: "".to_string(),
                amount: amount.0,
            };
            log!("{}", event);
        } else if transfer_msg.typ == 1 {
            assert!(self.valid_fungible_token_out(&token, MAP_CHAIN_ID )
                    || self.valid_mcs_token_out(&token, MAP_CHAIN_ID),
                    "deposit token {} to chain {} is not supported", token, MAP_CHAIN_ID);

            let order_id = self.get_order_id(&token,
                                             &from,
                                             &transfer_msg.to,
                                             amount.0,
                                             MAP_CHAIN_ID);
            let event = DepositOutEvent {
                from,
                to: transfer_msg.to,
                order_id,
                token,
                amount: amount.0,
            };
            log!("{}", event);
        } else {
            env::panic(format!("transfer msg typ {} is not supported", transfer_msg.typ).as_ref());
        }

        PromiseOrValue::Value(U128::from(0))
    }
}

admin_controlled::impl_admin_controlled!(MapCrossChainService, paused);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, MockedBlockchain};

    use super::*;
    use near_sdk::env::sha256;
    use std::convert::TryInto;
    use std::panic;
    use uint::rustc_hex::{FromHex, ToHex};
    use map_light_client::proof::LogEntry;

    const UNPAUSE_ALL: Mask = 0;

    macro_rules! inner_set_env {
        ($builder:ident) => {
            $builder
        };

        ($builder:ident, $key:ident:$value:expr $(,$key_tail:ident:$value_tail:expr)*) => {
            {
               $builder.$key($value.try_into().unwrap());
               inner_set_env!($builder $(,$key_tail:$value_tail)*)
            }
        };
    }

    macro_rules! set_env {
        ($($key:ident:$value:expr),* $(,)?) => {
            let mut builder = VMContextBuilder::new();
            let mut builder = &mut builder;
            builder = inner_set_env!(builder, $($key: $value),*);
            testing_env!(builder.build());
        };
    }

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn prover() -> AccountId {
        "prover".to_string()
    }

    fn bridge_token_factory() -> AccountId {
        "bridge".to_string()
    }

    fn token_locker() -> String {
        "6b175474e89094c44da98b954eedeac495271d0f".to_string()
    }

    /// Generate a valid ethereum address
    fn ethereum_address_from_id(id: u8) -> String {
        let mut buffer = vec![id];
        sha256(buffer.as_mut())
            .into_iter()
            .take(20)
            .collect::<Vec<_>>()
            .to_hex()
    }

    fn sample_proof() -> Proof {
        Proof {
            log_index: 0,
            log_entry_data: vec![],
            receipt_index: 0,
            receipt_data: vec![],
            header_data: vec![],
            proof: vec![],
        }
    }

    fn create_proof(locker: String, token: String) -> Proof {
        let event_data = MapTransferOutEvent {
            map_bridge_address: locker
                .from_hex::<Vec<_>>()
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),

            token,
            sender: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
            amount: 1000,
            recipient: "123".to_string(),
        };

        Proof {
            log_index: 0,
            log_entry_data: event_data.to_log_entry_data(),
            receipt_index: 0,
            receipt_data: vec![],
            header_data: vec![],
            proof: vec![],
        }
    }

    #[test]
    #[should_panic]
    fn test_fail_deploy_bridge_token() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());
        set_env!(
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE,
        );
        contract.deploy_bridge_token(token_locker());
    }

    #[test]
    #[should_panic]
    fn test_fail_deposit_no_token() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());
        set_env!(
            predecessor_account_id: alice(),
            attached_deposit: env::storage_byte_cost() * 1000
        );
        contract.deposit(sample_proof());
    }

    #[test]
    fn test_deploy_bridge_token() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2,
        );

        contract.deploy_bridge_token(token_locker());
        assert_eq!(
            contract.get_bridge_token_account_id(token_locker()),
            format!("{}.{}", token_locker(), bridge_token_factory())
        );

        let uppercase_address = "0f5Ea0A652E851678Ebf77B69484bFcD31F9459B".to_string();
        contract.deploy_bridge_token(uppercase_address.clone());
        assert_eq!(
            contract.get_bridge_token_account_id(uppercase_address.clone()),
            format!(
                "{}.{}",
                uppercase_address.to_lowercase(),
                bridge_token_factory()
            )
        );
    }

    #[test]
    fn test_finish_withdraw() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());

        set_env!(
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        contract.deploy_bridge_token(token_locker());

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: format!("{}.{}", token_locker(), bridge_token_factory())
        );

        let address = validate_eth_address(token_locker());
        assert_eq!(
            contract.finish_withdraw(1_000, token_locker()),
            ResultType::Withdraw {
                amount: 1_000,
                token: address,
                recipient: address,
            }
        );
    }

    #[test]
    fn deploy_bridge_token_paused() {
        set_env!(predecessor_account_id: alice());

        // User alice can deploy a new bridge token
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        contract.deploy_bridge_token(ethereum_address_from_id(0));

        // Admin pause deployment of new token
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        contract.set_paused(PAUSE_DEPLOY_TOKEN);

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        // Admin can still deploy new tokens after paused
        contract.deploy_bridge_token(ethereum_address_from_id(1));

        // User alice can't deploy a new bridge token when it is paused
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        panic::catch_unwind(move || {
            contract.deploy_bridge_token(ethereum_address_from_id(2));
        })
            .unwrap_err();
    }

    #[test]
    fn only_admin_can_pause() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());

        // Admin can pause
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
        );
        contract.set_paused(0b1111);

        // Alice can't pause
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
        );

        panic::catch_unwind(move || {
            contract.set_paused(0);
        })
            .unwrap_err();
    }

    #[test]
    fn deposit_paused() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        let erc20_address = ethereum_address_from_id(0);
        contract.deploy_bridge_token(erc20_address.clone());

        // Check it is possible to use deposit while the contract is NOT paused
        contract.deposit(create_proof(token_locker(), erc20_address.clone()));

        // Pause deposit
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        contract.set_paused(PAUSE_TRANSFER_IN);

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );

        // Check it is NOT possible to use deposit while the contract is paused
        panic::catch_unwind(move || {
            contract.deposit(create_proof(token_locker(), erc20_address.clone()));
        })
            .unwrap_err();
    }

    /// Check after all is paused deposit is not available
    #[test]
    fn all_paused() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        let erc20_address = ethereum_address_from_id(0);
        contract.deploy_bridge_token(erc20_address.clone());

        // Check it is possible to use deposit while the contract is NOT paused
        contract.deposit(create_proof(token_locker(), erc20_address.clone()));

        // Pause everything
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        contract.set_paused(PAUSE_DEPLOY_TOKEN | PAUSE_TRANSFER_IN);

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );

        // Check it is NOT possible to use deposit while the contract is paused
        panic::catch_unwind(move || {
            contract.deposit(create_proof(token_locker(), erc20_address));
        })
            .unwrap_err();
    }

    /// Check after all is paused and unpaused deposit works
    #[test]
    fn no_paused() {
        set_env!(predecessor_account_id: alice());
        let mut contract = BridgeTokenFactory::new(prover(), token_locker());

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );
        let erc20_address = ethereum_address_from_id(0);
        contract.deploy_bridge_token(erc20_address.clone());

        // Check it is possible to use deposit while the contract is NOT paused
        contract.deposit(create_proof(token_locker(), erc20_address.clone()));

        // Pause everything
        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: bridge_token_factory(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );

        contract.set_paused(PAUSE_DEPLOY_TOKEN | PAUSE_TRANSFER_IN);
        contract.set_paused(UNPAUSE_ALL);

        set_env!(
            current_account_id: bridge_token_factory(),
            predecessor_account_id: alice(),
            attached_deposit: BRIDGE_TOKEN_INIT_BALANCE * 2
        );

        // Check the deposit works after pausing and unpausing everything
        contract.deposit(create_proof(token_locker(), erc20_address));
    }
}
