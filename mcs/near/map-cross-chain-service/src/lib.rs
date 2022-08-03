use std::collections::HashSet;
use admin_controlled::{AdminControlled, Mask};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PublicKey, PromiseOrValue, PromiseResult, CryptoHash, log};
use event::*;
use prover::*;
use std::str::FromStr;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::env::panic_str;
use map_light_client::proof::ReceiptProof;

mod event;
pub mod prover;
mod bytes;

const MCS_TOKEN_BINARY: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/mcs_token.wasm");

const MAP_CHAIN_ID: u128 = 22776;

const NO_DEPOSIT: Balance = 0;

/// Initial balance for the MCSToken contract to cover storage and related.
const MCS_TOKEN_INIT_BALANCE: Balance = 5_000_000_000_000_000_000_000_000; // 5e24yN, 5N

/// Gas to initialize MCSToken contract.
const MCS_TOKEN_NEW: Gas = Gas(10_000_000_000_000);

/// Gas to call ft_transfer on ext fungible contract
const FT_TRANSFER_GAS: Gas = Gas(80_000_000_000_000);

/// Gas to call mint method on mcs token.
const MINT_GAS: Gas = Gas(10_000_000_000_000);

/// Gas to call burn method on mcs token.
const BURN_GAS: Gas = Gas(10_000_000_000_000);

/// Gas to call near_withdraw when the to chain token address is empty
const NEAR_WITHDRAW_GAS: Gas = Gas(40_000_000_000_000);

/// Gas to call near_deposit when the to chain token address is empty
const NEAR_DEPOSIT_GAS: Gas = Gas(40_000_000_000_000);

/// Gas to call finish deposit method.
/// This doesn't cover the gas required for calling mint method.
const FINISH_TRANSFER_IN_SINGLE_EVENT_GAS: Gas = Gas(80_000_000_000_000);

/// Gas to call finish transfer out method.
const FINISH_TRANSFER_OUT_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish transfer in native token method.
const FINISH_TRANSFER_IN_NATIVE_TOKEN_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = Gas(50_000_000_000_000);

/// Amount of gas used by set_metadata in the mcs, without taking into account
/// the gas consumed by the promise.
const OUTER_SET_METADATA_GAS: Gas = Gas(15_000_000_000_000);

/// Controller storage key.
const CONTROLLER_STORAGE_KEY: &[u8] = b"aCONTROLLER";

const TRANSFER_OUT_TYPE: &str = "2ef1cdf83614a69568ed2c96a275dd7fb2e63a464aa3a0ffe79f55d538c8b3b5";
const DEPOSIT_OUT_TYPE: &str = "150bd848adaf4e3e699dcac82d75f111c078ce893375373593cc1b9208998377";

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
    /// Set of other fungible token contracts.
    pub native_to_chains: HashSet<u128>,
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<CryptoHash>,
    /// Public key of the account deploying the MCS contract.
    pub owner_pk: PublicKey,
    /// Balance required to register a new account in the MCSToken
    pub mcs_storage_transfer_in_required: Balance,
    // Wrap token for near
    // mainnet: wrap.near, testnet:wrap.testnet
    pub wrapped_token: String,
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
        #[serializer(borsh)]
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
        receiver_id: AccountId,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMsg {
    pub typ: u8,
    // 0: Transfer or 1: Deposit
    pub to: String,
    pub to_chain: u128, // if typ is 1, it is omitted
}

#[near_bindgen]
impl MapCrossChainService {
    /// Initializes the contract.
    /// map_client_account: NEAR account of the MAP light client contract;
    /// map_bridge_address: the address of the MCS contract on MAP blockchain, in hex.
    /// wrapped_token: the wrap near contract account id
    /// near_chain_id: the chain id of the near blockchain
    #[init]
    pub fn init(map_light_client: String, map_bridge_address: String, wrapped_token: String, near_chain_id: u128) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        Self {
            map_client_account: map_light_client.parse().unwrap(),
            map_bridge_address: validate_eth_address(map_bridge_address),
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            native_to_chains: Default::default(),
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

    pub fn version() -> &'static str {
        "0.1.1"
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

        ext_map_light_client::ext(self.map_client_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_proof_data(receipt_proof)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_TRANSFER_IN_SINGLE_EVENT_GAS * event_len)
                    .with_attached_deposit(env::attached_deposit())
                    .finish_transfer_in(events)
            )
    }

    pub fn transfer_out_token(&mut self, token: String, to: String, amount: u128, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_TOKEN);

        if self.valid_mcs_token_out(&token, to_chain) {
            let from = env::signer_account_id().to_string();
            let token = self.get_mcs_token_account_id(token.clone()).to_string();
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

            ext_mcs_token::ext(event.token.parse().unwrap())
                .with_static_gas(BURN_GAS)
                .burn(from.parse().unwrap(), event.amount.into())
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TRANSFER_OUT_GAS)
                        .finish_transfer_out(event)
                )
        } else if self.valid_fungible_token_out(&token, to_chain) {
            env::panic_str(format!("non mcs fungible token {} should called from fungible token directly", token).as_ref());
        } else {
            env::panic_str(format!("token {} to chain {} is not supported", token, to_chain).as_ref());
        }
    }

    #[payable]
    pub fn transfer_out_native(&mut self, to: String, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");
        assert!(self.native_to_chains.contains(&to_chain), "transfer out native to {} is not supported", to_chain);

        let from = env::signer_account_id().to_string();
        let order_id = self.get_order_id(&"".to_string(), &from, &to, amount, to_chain);

        let event = TransferOutEvent {
            from_chain: self.near_chain_id,
            to_chain,
            from: from.to_string(),
            to: to.parse().unwrap(),
            order_id,
            token: "".to_string(),
            to_chain_token: "".to_string(),
            amount,
        };

        ext_wnear_token::ext(self.wrapped_token.parse().unwrap())
            .with_static_gas(NEAR_DEPOSIT_GAS)
            .with_attached_deposit(env::attached_deposit())
            .near_deposit()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_TRANSFER_OUT_GAS)
                    .finish_transfer_out(event)
            )
    }

    /// Finish transfer in once the proof was successfully validated. Can only be called by the contract
    /// itself.
    #[payable]
    pub fn finish_transfer_in(
        &mut self,
        events: Vec<MapTransferOutEvent>,
    ) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get result from cross contract");

        assert_self();

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
        assert!(cur_deposit >= required_deposit, "not enough deposit for record proof, exp: {}, cur: {}", required_deposit, cur_deposit);

        env::log_str(&*format!("transfer in to: {}", event.to));

        let mut ret_deposit = cur_deposit - required_deposit;

        if event.to_chain_token == "" {
            assert!(ret_deposit >= 1, "not enough deposit for near withdraw");
            ret_deposit = ret_deposit - 1;

            (ext_wnear_token::ext(self.wrapped_token.parse().unwrap())
                 .with_static_gas(NEAR_WITHDRAW_GAS)
                 .with_attached_deposit(1)
                 .near_withdraw(event.amount.into())
                 .then(
                     Self::ext(env::current_account_id())
                         .with_static_gas(FINISH_TRANSFER_IN_NATIVE_TOKEN_GAS)
                         .finish_transfer_in_native_token(event.clone())
                 ), ret_deposit)
        } else if self.mcs_tokens.get(&event.to_chain_token).is_some() {
            assert!(ret_deposit >= self.mcs_storage_transfer_in_required, "not enough deposit for mcs token mint");
            ret_deposit = ret_deposit - self.mcs_storage_transfer_in_required;

            (ext_mcs_token::ext(self.get_mcs_token_account_id(event.token.clone()))
                 .with_static_gas(MINT_GAS)
                 .with_attached_deposit(self.mcs_storage_transfer_in_required)
                 .mint(event.to.clone().parse().unwrap(), event.amount.into()), ret_deposit)
        } else if self.fungible_tokens.get(&event.to_chain_token).is_some() {
            assert!(ret_deposit >= 1, "not enough deposit for ft transfer");
            ret_deposit = ret_deposit - 1;

            (ext_fungible_token::ext(event.to_chain_token.parse().unwrap())
                 .with_static_gas(FT_TRANSFER_GAS)
                 .with_attached_deposit(1)
                 .ft_transfer(event.to.clone().parse().unwrap(), event.amount.into(), None), ret_deposit)
        } else {
            panic_str(&*format!("unknown to_chain_token {} to transfer in", event.to_chain_token))
        }
    }

    pub fn finish_transfer_in_native_token(
        &mut self,
        event: &MapTransferOutEvent
    ) -> Promise {
        assert_self();
        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get result from cross contract");
        Promise::new(event.to.clone().parse().unwrap()).transfer(event.amount.into())
    }

    /// Finish transfer out once the nep141 token is burned from MCSToken contract or native token is transferred.
    pub fn finish_transfer_out(
        &mut self,
        #[serializer(borsh)]
        event: TransferOutEvent,
    ) {
        assert_self();
        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get result from cross contract");
        log!("transfer out: {}", serde_json::to_string(&event).unwrap());
        log!("{}{}", TRANSFER_OUT_TYPE, event);
    }

    #[payable]
    pub fn deposit_out_native(&mut self, to: String) {
        self.check_not_paused(PAUSE_DEPOSIT_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");

        let from = env::signer_account_id().to_string();

        let order_id = self.get_order_id(&"".to_string(), &from, &to, amount, MAP_CHAIN_ID);

        let event = DepositOutEvent {
            token: "".to_string(),
            from,
            to,
            order_id,
            amount,
        };

        log!("deposit out: {}", serde_json::to_string(&event).unwrap());
        log!("{}{}", DEPOSIT_OUT_TYPE, event);
    }

    #[payable]
    pub fn deploy_mcs_token(&mut self, address: String) -> Promise {
        self.check_not_paused(PAUSE_DEPLOY_TOKEN);
        let address = address.to_lowercase();
        // let _ = validate_eth_address(address.clone());
        assert!(
            self.mcs_tokens.get(&address).is_none(),
            "MCS token contract already exists."
        );

        assert!(
            self.fungible_tokens.get(&address).is_none(),
            "Fungible token contract with same name exists."
        );
        let initial_storage = env::storage_usage() as u128;
        self.mcs_tokens.insert(&address, &Default::default());
        let current_storage = env::storage_usage() as u128;
        assert!(
            env::attached_deposit()
                >= MCS_TOKEN_INIT_BALANCE
                + env::storage_byte_cost() * (current_storage - initial_storage),
            "Not enough attached deposit to complete mcs token creation"
        );

        let mcs_token_account_id = format!("{}.{}", address, env::current_account_id());
        Promise::new(mcs_token_account_id.parse().unwrap())
            .create_account()
            .transfer(MCS_TOKEN_INIT_BALANCE)
            .add_full_access_key(self.owner_pk.clone())
            .deploy_contract(MCS_TOKEN_BINARY.to_vec())
            .function_call(
                "new".to_string(),
                b"{}".to_vec(),
                NO_DEPOSIT,
                MCS_TOKEN_NEW,
            )
    }

    pub fn get_mcs_token_account_id(&self, address: String) -> AccountId {
        let address = address.to_lowercase();
        assert!(
            self.mcs_tokens.get(&address).is_some(),
            "MCSToken with such address does not exist."
        );

        AccountId::from_str(&*format!("{}.{}", address, env::current_account_id())).unwrap()
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

        env::log_str(&*format!("RecordOrderId:{}", hex::encode(order_id)));
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

        ext_mcs_token::ext(address.parse().unwrap())
            .with_static_gas(env::prepaid_gas() - OUTER_SET_METADATA_GAS)
            .with_attached_deposit(env::attached_deposit())
            .set_metadata(
                name,
                symbol,
                reference,
                reference_hash,
                decimals,
                icon,
            )
    }

    /// Factory Controller. Controller has extra privileges inside this contract.
    pub fn controller(&self) -> Option<AccountId> {
        env::storage_read(CONTROLLER_STORAGE_KEY)
            .map(|value| String::from_utf8(value).expect("Invalid controller account id").parse().unwrap())
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

    pub fn add_native_to_chain(&mut self, to_chain: u128) {
        assert!(self.controller_or_self());

        self.native_to_chains.insert(to_chain);
    }

    pub fn remove_native_to_chain(&mut self, to_chain: u128) {
        assert!(self.controller_or_self());

        self.native_to_chains.remove(&to_chain);
    }

    pub fn valid_mcs_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.mcs_tokens.get(&token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    pub fn add_fungible_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());
        assert!(self.mcs_tokens.get(&token).is_none(), "token name {} exists in mcs token", token);

        let mut to_chain_set = self.fungible_tokens.get(&token).unwrap_or(Default::default());
        to_chain_set.insert(to_chain);
        self.fungible_tokens.insert(&token, &to_chain_set);
    }

    pub fn remove_fungible_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());

        let mut to_chain_set = self.fungible_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain);
        self.fungible_tokens.insert(&token, &to_chain_set);
    }

    pub fn valid_fungible_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.fungible_tokens.get(&token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    pub fn add_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.insert(to_chain);
        self.mcs_tokens.insert(&token, &to_chain_set);
    }

    pub fn remove_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain);
        self.mcs_tokens.insert(&token, &to_chain_set);
    }

    fn get_order_id(&mut self, token: &String, from: &String, to: &String, amount: u128, to_chain_id: u128) -> CryptoHash {
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

    // FIXME: for testing only
    // #[payable]
    // pub fn mint(&mut self, token: String, to: String, amount: u128) -> Promise {
    //     ext_mcs_token::ext( self.get_mcs_token_account_id(token))
    //         .with_static_gas(MINT_GAS)
    //         .with_attached_deposit(self.mcs_storage_transfer_in_required + 2)
    //         .mint(to.parse().unwrap(), U128::from(amount))
    // }
}

#[near_bindgen]
impl FungibleTokenReceiver for MapCrossChainService {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id().to_string();
        let transfer_msg: FungibleTokenMsg = serde_json::from_str(&msg).unwrap();

        let from = sender_id.to_string();
        if transfer_msg.typ == 0 {
            self.check_not_paused(PAUSE_TRANSFER_OUT_TOKEN);
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
            log!("transfer out: {}", serde_json::to_string(&event).unwrap());
            log!("{}{}", TRANSFER_OUT_TYPE, event);
        } else if transfer_msg.typ == 1 {
            self.check_not_paused(PAUSE_DEPOSIT_OUT_TOKEN);
            assert!(self.valid_fungible_token_out(&token, MAP_CHAIN_ID)
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
            log!("deposit out: {}", serde_json::to_string(&event).unwrap());
            log!("{}{}", DEPOSIT_OUT_TYPE, event);
        } else {
            panic_str(format!("transfer msg typ {} is not supported", transfer_msg.typ).as_ref());
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
    use map_light_client::header::{Hash, Header};
    use map_light_client::proof::{LogEntry, Receipt};
    use map_light_client::G2;

    const UNPAUSE_ALL: Mask = 0;
    const NEAR_CHAIN_ID: u128 = 1313161555;
    const ETH_CHAIN_ID: u128 = 1;

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

    fn controller() -> String {
        "map.near".to_string()
    }

    fn alice() -> (AccountId, String) {
        ("alice.near".parse().unwrap(), "ab175474e89094c44da98b954eedeac495271d0f".to_string())
    }

    fn prover() -> String {
        "prover".to_string()
    }

    fn map_cross_chain_service() -> AccountId {
        "mcs".parse().unwrap()
    }

    fn map_bridge_address() -> String {
        "6b175474e89094c44da98b954eedeac495271d0f".to_string()
    }

    fn wrap_token() -> String {
        "wrap.testnet".to_string()
    }

    fn mcs_token() -> (String, String) {
        ("mcs".to_string(), "cb175474e89094c44da98b954eedeac495271d0f".to_string())
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

    fn sample_proof() -> ReceiptProof {
        ReceiptProof {
            header: Header::new(),
            agg_pk: G2 {
                xr: [0; 32],
                xi: [0; 32],
                yr: [0; 32],
                yi: [0; 32],
            },
            receipt: Receipt {
                receipt_type: 0,
                post_state_or_status: vec![],
                cumulative_gas_used: 0,
                bloom: [0; 256],
                logs: vec![],
            },
            key_index: vec![],
            proof: vec![],
        }
    }

    // fn create_proof(locker: String, token: String) -> Proof {
    //     let event_data = MapTransferOutEvent {
    //         map_bridge_address: locker
    //             .from_hex::<Vec<_>>()
    //             .unwrap()
    //             .as_slice()
    //             .try_into()
    //             .unwrap(),
    //
    //         token,
    //         sender: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
    //         amount: 1000,
    //         recipient: "123".to_string(),
    //     };
    //
    //     Proof {
    //         log_index: 0,
    //         log_entry_data: event_data.to_log_entry_data(),
    //         receipt_index: 0,
    //         receipt_data: vec![],
    //         header_data: vec![],
    //         proof: vec![],
    //     }
    // }

    #[test]
    #[should_panic]
    fn test_fail_deploy_mcs_token() {
        set_env!(predecessor_account_id: alice().0);
        let mut contract = MapCrossChainService::init(prover(),
                                                      map_bridge_address(),
                                                      wrap_token(),
                                                      NEAR_CHAIN_ID);
        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE,
        );
        contract.deploy_mcs_token(map_bridge_address());
    }

    #[test]
    #[should_panic]
    fn test_fail_transfer_in_no_event() {
        set_env!(predecessor_account_id: alice().0);
        let mut contract = MapCrossChainService::init(prover(),
                                                      map_bridge_address(),
                                                      wrap_token(),
                                                      NEAR_CHAIN_ID);
        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: env::storage_byte_cost() * 1000
        );
        contract.transfer_in(sample_proof());
    }

    #[test]
    fn test_deploy_mcs_token() {
        set_env!(predecessor_account_id: alice().0);
        let mut contract = MapCrossChainService::init(prover(),
                                                      map_bridge_address(),
                                                      wrap_token(),
                                                      NEAR_CHAIN_ID);
        set_env!(
            current_account_id: map_cross_chain_service(),
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE * 2,
        );

        contract.deploy_mcs_token(map_bridge_address());
        assert_eq!(
            contract.get_mcs_token_account_id(map_bridge_address()).to_string(),
            format!("{}.{}", map_bridge_address(), map_cross_chain_service())
        );

        let uppercase_address = "0f5Ea0A652E851678Ebf77B69484bFcD31F9459B".to_string();
        contract.deploy_mcs_token(uppercase_address.clone());
        assert_eq!(
            contract.get_mcs_token_account_id(uppercase_address.clone()).to_string(),
            format!(
                "{}.{}",
                uppercase_address.to_lowercase(),
                map_cross_chain_service()
            )
        );
    }

    #[test]
    #[should_panic]
    fn test_transfer_out_token_no_to_chain() {
        let token = mcs_token().0;
        let from = alice().0;
        let to = alice().1;

        set_env!(predecessor_account_id: controller());
        let mut contract = MapCrossChainService::init(prover(),
                                                      map_bridge_address(),
                                                      wrap_token(),
                                                      NEAR_CHAIN_ID);

        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE * 2
        );
        contract.deploy_mcs_token(mcs_token().0);

        set_env!(
            current_account_id: map_cross_chain_service(),
            predecessor_account_id: format!("{}.{}", token, map_cross_chain_service())
        );

        contract.transfer_out_token(token, to, 1_000, ETH_CHAIN_ID);
    }
}
