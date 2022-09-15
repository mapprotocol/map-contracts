use std::collections::HashSet;
use admin_controlled::{AdminControlled, Mask};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PublicKey, PromiseOrValue, PromiseResult, CryptoHash, log};
use event::*;
use prover::*;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
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
const FT_TRANSFER_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call storage_deposit on ext fungible contract
const STORAGE_DEPOSIT_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call ft_metadata on ext fungible contract
const FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call storage_balance_bounds on ext fungible contract
const STORAGE_BALANCE_BOUNDS_GAS: Gas = Gas(30_000_000_000_000);

const GET_FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call finish_init on mcs contract
const FINISH_INIT_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish_add_fungible_token_to_chain method.
const FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS: Gas = Gas(20_000_000_000_000);

/// Gas to call mint/burn method on mcs token.
const MINT_GAS: Gas = Gas(10_000_000_000_000);
const BURN_GAS: Gas = Gas(10_000_000_000_000);

/// Gas to call near_withdraw and near_deposit on wrap near contract
const NEAR_WITHDRAW_GAS: Gas = Gas(20_000_000_000_000);
const NEAR_DEPOSIT_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call storage_deposit_for_mcs on mcs contract
const STORAGE_DEPOSIT_FOR_MCS_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call finish_verify_proof method.
const TRANSFER_IN_SINGLE_EVENT_GAS: Gas = Gas(60_000_000_000_000);

/// Gas to call transfer_in_native_token method.
const TRANSFER_IN_NATIVE_TOKEN_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish_transfer_in method.
const FINISH_TRANSFER_IN_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish_transfer_out method.
const FINISH_TRANSFER_OUT_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call report_fail method.
const REPORT_FAIL_GAS: Gas = Gas(10_000_000_000_000);

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = Gas(40_000_000_000_000);

/// Amount of gas used by set_metadata in the mcs, without taking into account
/// the gas consumed by the promise.
const OUTER_SET_METADATA_GAS: Gas = Gas(15_000_000_000_000);

/// Controller storage key.
const CONTROLLER_STORAGE_KEY: &[u8] = b"aCONTROLLER";

const TRANSFER_OUT_TYPE: &str = "2ef1cdf83614a69568ed2c96a275dd7fb2e63a464aa3a0ffe79f55d538c8b3b5";
const DEPOSIT_OUT_TYPE: &str = "150bd848adaf4e3e699dcac82d75f111c078ce893375373593cc1b9208998377";

const MIN_TRANSFER_OUT_AMOUNT: f64 = 0.001;
const NEAR_DECIMAL: u8 = 24;

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
    /// Map of other fungible token contracts and their min storage balance.
    pub fungible_tokens_storage_balance: UnorderedMap<String, u128>,
    /// Map of token contracts and their decimals
    pub token_decimals: UnorderedMap<String, u8>,
    /// Set of other fungible token contracts.
    pub native_to_chains: HashSet<u128>,
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<CryptoHash>,
    /// Public key of the account deploying the MCS contract.
    pub owner_pk: PublicKey,
    /// Balance required to register a new account in the MCSToken
    pub mcs_storage_transfer_in_required: Balance,
    // Wrap token for near
    pub wrapped_token: String,
    // Near chain id
    // FIXME: get from env?
    pub near_chain_id: u128,
    // Nonce to generate order id
    pub nonce: u128,
    /// Mask determining all paused functions
    paused: Mask,
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance;
    fn storage_balance_bounds(&self) -> StorageBalanceBounds;
    fn ft_metadata(&self) -> FungibleTokenMetadata;
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
    assert_eq!(env::predecessor_account_id(), env::current_account_id(), "caller is not self");
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMsg {
    pub msg_type: u8,
    // 0: Transfer or 1: Deposit
    pub to: Vec<u8>,
    pub to_chain: u128, // if msg_type is 1, it is omitted
}

#[near_bindgen]
impl MapCrossChainService {
    /// Initializes the contract.
    /// `map_client_account`: NEAR account of the MAP light client contract;
    /// `map_bridge_address`: the address of the MCS contract on MAP blockchain, in hex.
    /// `wrapped_token`: the wrap near contract account id
    /// `near_chain_id`: the chain id of the near blockchain
    pub fn init(map_light_client: String, map_bridge_address: String, wrapped_token: String, near_chain_id: u128) -> Promise {
        assert!(!env::state_exists(), "Already initialized");

        let storage_balance = near_contract_standards::fungible_token::FungibleToken::new(b"t".to_vec())
            .account_storage_usage as Balance * env::storage_byte_cost();

        ext_fungible_token::ext(wrapped_token.parse().unwrap())
            .with_static_gas(STORAGE_DEPOSIT_GAS)
            .with_attached_deposit(storage_balance)
            .storage_deposit(Some(env::current_account_id()), Some(true))
            .then(Self::ext(env::current_account_id())
                .with_static_gas(FINISH_INIT_GAS)
                .finish_init(map_light_client, map_bridge_address, wrapped_token, near_chain_id, storage_balance))
    }
    #[init]
    pub fn finish_init(map_light_client: String, map_bridge_address: String, wrapped_token: String, near_chain_id: u128, storage_balance: u128) -> Self {
        assert_self();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        let _balance = match env::promise_result(0) {
            PromiseResult::Successful(x) => serde_json::from_slice::<StorageBalance>(&x).unwrap(),
            _ => panic_str("wnear contract storage deposit failed"),
        };

        Self {
            map_client_account: map_light_client.parse().unwrap(),
            map_bridge_address: validate_eth_address(map_bridge_address),
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            fungible_tokens_storage_balance: UnorderedMap::new(b"s".to_vec()),
            token_decimals: UnorderedMap::new(b"d".to_vec()),
            native_to_chains: Default::default(),
            used_events: UnorderedSet::new(b"u".to_vec()),
            owner_pk: env::signer_account_pk(),
            mcs_storage_transfer_in_required: storage_balance,
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
    pub fn transfer_in(&mut self, receipt_proof: ReceiptProof, index: usize) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_IN);

        assert!(index < receipt_proof.receipt.logs.len(), "index exceeds event size");
        let event_opt = MapTransferOutEvent::from_log_entry_data(receipt_proof.receipt.logs.get(index).unwrap());
        assert!(event_opt.is_some(), "not map transfer out event");
        let event = event_opt.unwrap();
        assert_eq!(self.map_bridge_address, event.map_bridge_address, "unexpected map mcs address: {}", hex::encode(event.map_bridge_address));

        let to_chain_token = String::from_utf8(event.to_chain_token.clone()).unwrap();
        log!("get transfer in event: {}", event);
        assert_eq!(self.near_chain_id, event.to_chain, "unexpected to chain: {}", event.to_chain);
        assert!(self.mcs_tokens.get(&to_chain_token).is_some()
                    || self.fungible_tokens.get(&to_chain_token).is_some() || self.is_native_token(event.to_chain_token.clone()),
                "to_chain_token {} is not mcs token or fungible token or empty", to_chain_token);
        assert_eq!(false, self.is_used_event(&event.order_id), "the event with order id {} is used", hex::encode(event.order_id));

        ext_map_light_client::ext(self.map_client_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_proof_data(receipt_proof)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(TRANSFER_IN_SINGLE_EVENT_GAS + FINISH_TRANSFER_IN_GAS)
                    .with_attached_deposit(env::attached_deposit())
                    .finish_verify_proof(&event)
            )
    }

    pub fn transfer_out_token(&mut self, token: String, to: Vec<u8>, amount: U128, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_TOKEN);

        if self.valid_mcs_token_out(&token, to_chain) {
            self.check_amount(token.clone(), amount.0, false);
            let from = env::signer_account_id().to_string();
            let amount = amount.0;
            let order_id = self.get_order_id(&token, &from, &to, amount, to_chain);

            let event = TransferOutEvent {
                from_chain: self.near_chain_id,
                to_chain,
                from: from.clone(),
                to,
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
    pub fn transfer_out_native(&mut self, to: Vec<u8>, to_chain: u128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");
        assert!(self.native_to_chains.contains(&to_chain), "transfer out native to {} is not supported", to_chain);
        self.check_amount("".to_string(), amount, true);

        let from = env::signer_account_id().to_string();
        let order_id = self.get_order_id(&self.native_token_address().1, &from, &to, amount, to_chain);

        let event = TransferOutEvent {
            from_chain: self.near_chain_id,
            to_chain,
            from: from.to_string(),
            to,
            order_id,
            token: self.native_token_address().1,
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
    pub fn finish_verify_proof(
        &mut self,
        event: &MapTransferOutEvent,
    ) -> Promise {
        assert_self();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Failed => {
                Promise::new(env::signer_account_id()).transfer(env::attached_deposit())
                    .then(Self::ext(env::current_account_id())
                        .with_static_gas(REPORT_FAIL_GAS)
                        .report_transfer_in_fail("verify proof failed".to_string()))
            }
            PromiseResult::Successful(_) => { self.process_transfer_in(event) }
        }
    }

    fn process_transfer_in(&mut self, event: &MapTransferOutEvent) -> Promise {
        let cur_deposit = env::attached_deposit();
        let required_deposit = self.record_order_id(&event.order_id);
        if cur_deposit < required_deposit {
            return self.process_transfer_in_failure(cur_deposit, event,
                                                    format!("not enough deposit for record proof, exp: {}, cur: {}", required_deposit, cur_deposit));
        }

        let to = String::from_utf8(event.to.clone()).unwrap();
        let to_chain_token = String::from_utf8(event.to_chain_token.clone()).unwrap();
        env::log_str(&*format!("start to transfer in token: {}, to: {}, amount: {}", to_chain_token, to, event.amount));

        let mut ret_deposit = cur_deposit - required_deposit;

        if self.is_native_token(event.to_chain_token.clone()) {
            if ret_deposit < 1 {
                return self.process_transfer_in_failure(cur_deposit, event,
                                                        "not enough deposit for near withdraw".to_string());
            }

            ext_wnear_token::ext(self.wrapped_token.parse().unwrap())
                .with_static_gas(NEAR_WITHDRAW_GAS)
                .with_attached_deposit(1)
                .near_withdraw(event.amount.into())
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(TRANSFER_IN_NATIVE_TOKEN_GAS)
                        .with_attached_deposit(ret_deposit)
                        .transfer_in_native_token(event)
                )
        } else if self.mcs_tokens.get(&to_chain_token).is_some() {
            if ret_deposit < self.mcs_storage_transfer_in_required {
                return self.process_transfer_in_failure(cur_deposit, event,
                                                        format!("not enough deposit for mcs token mint, exp: {}, cur: {}", self.mcs_storage_transfer_in_required, ret_deposit));
            }
            ret_deposit = ret_deposit - self.mcs_storage_transfer_in_required;

            ext_mcs_token::ext(to_chain_token.parse().unwrap())
                .with_static_gas(MINT_GAS)
                .with_attached_deposit(self.mcs_storage_transfer_in_required)
                .mint(to.parse().unwrap(), event.amount.into())
                .then(Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_TRANSFER_IN_GAS)
                    .with_attached_deposit(ret_deposit)
                    .finish_transfer_in(event))
        } else if self.fungible_tokens.get(&to_chain_token).is_some() {
            let min_storage_balance = self.fungible_tokens_storage_balance.get(&to_chain_token).unwrap();
            if ret_deposit < min_storage_balance + 1 {
                return self.process_transfer_in_failure(cur_deposit, event,
                                                        format!("not enough deposit for ft transfer, exp: {}, cur: {}", 1 + min_storage_balance, ret_deposit));
            }
            ret_deposit = ret_deposit - 1 - min_storage_balance;

            let token_account: AccountId = to_chain_token.parse().unwrap();
            ext_fungible_token::ext(token_account.clone())
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .with_attached_deposit(min_storage_balance)
                .storage_deposit(Some(to.parse().unwrap()), Some(true))
                .then(ext_fungible_token::ext(token_account)
                    .with_static_gas(FT_TRANSFER_GAS)
                    .with_attached_deposit(1)
                    .ft_transfer(to.parse().unwrap(), event.amount.into(), None))
                .then(Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_TRANSFER_IN_GAS)
                    .with_attached_deposit(ret_deposit)
                    .finish_transfer_in(event))
        } else {
            self.process_transfer_in_failure(cur_deposit, event,
                                             format!("unknown to_chain_token {} to transfer in", to_chain_token))
        }
    }

    fn process_transfer_in_failure(&mut self, ret_deposit: Balance, event: &MapTransferOutEvent, err_msg: String) -> Promise {
        self.remove_order_id(&event.order_id);
        Promise::new(env::signer_account_id()).transfer(ret_deposit)
            .then(Self::ext(env::current_account_id())
                .with_static_gas(REPORT_FAIL_GAS)
                .report_transfer_in_fail(err_msg))
    }

    #[payable]
    pub fn transfer_in_native_token(
        &mut self,
        event: &MapTransferOutEvent,
    ) -> Promise {
        assert_self();
        assert_eq!(1, env::promise_results_count(), "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => {
                let to = String::from_utf8(event.to.clone()).unwrap();
                Promise::new(to.parse().unwrap()).transfer(event.amount.into())
                    .then(Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TRANSFER_IN_GAS)
                        .with_attached_deposit(env::attached_deposit())
                        .finish_transfer_in(event))
            }
            _ => {
                Promise::new(env::signer_account_id()).transfer(env::attached_deposit() + self.remove_order_id(&event.order_id))
                    .then(Self::ext(env::current_account_id())
                        .with_static_gas(REPORT_FAIL_GAS)
                        .report_transfer_in_fail("near withdraw failed".to_string()))
            }
        }
    }

    #[payable]
    pub fn finish_transfer_in(
        &mut self,
        event: &MapTransferOutEvent,
    ) -> Promise {
        assert_self();
        assert_eq!(1, env::promise_results_count(), "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => { Promise::new(env::signer_account_id()).transfer(env::attached_deposit()) }
            _ => {
                let mut promise = Promise::new(env::signer_account_id()).transfer(env::attached_deposit() + self.remove_order_id(&event.order_id));
                if self.is_native_token(event.to_chain_token.clone()) {
                    promise = promise.then(ext_wnear_token::ext(self.wrapped_token.parse().unwrap())
                        .with_static_gas(NEAR_DEPOSIT_GAS)
                        .with_attached_deposit(event.amount.into())
                        .near_deposit())
                }
                promise.then(Self::ext(env::current_account_id())
                    .with_static_gas(REPORT_FAIL_GAS)
                    .report_transfer_in_fail("transfer in token failed".to_string()))
            }
        }
    }

    pub fn report_transfer_in_fail(err: String) {
        assert_self();
        panic_str(err.as_str())
    }

    /// Finish transfer out once the nep141 token is burned from MCSToken contract or native token is transferred.
    pub fn finish_transfer_out(
        &self,
        event: TransferOutEvent,
    ) {
        assert_self();
        assert_eq!(PromiseResult::Successful(vec![]), env::promise_result(0), "get failed result from cross contract");
        log!("transfer out: {}", serde_json::to_string(&event).unwrap());
        log!("{}{}", TRANSFER_OUT_TYPE, event);
    }

    #[payable]
    pub fn deposit_out_native(&mut self, to: Vec<u8>) {
        self.check_not_paused(PAUSE_DEPOSIT_OUT_NATIVE);

        let amount = env::attached_deposit();
        assert!(amount > 0, "amount should > 0");
        self.check_amount("".to_string(), amount, true);

        let from = env::signer_account_id().to_string();

        let order_id = self.get_order_id(&self.native_token_address().1, &from, &to, amount, MAP_CHAIN_ID);

        let event = DepositOutEvent {
            token: self.native_token_address().1,
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
        let address = format!("{}.{}", address.to_lowercase(), env::current_account_id());
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
                >= MCS_TOKEN_INIT_BALANCE + self.mcs_storage_transfer_in_required
                + env::storage_byte_cost() * (current_storage - initial_storage),
            "Not enough attached deposit to complete mcs token creation"
        );

        Promise::new(address.parse().unwrap())
            .create_account()
            .transfer(MCS_TOKEN_INIT_BALANCE)
            .add_full_access_key(self.owner_pk.clone())
            .deploy_contract(MCS_TOKEN_BINARY.to_vec())
            .function_call(
                "new".to_string(),
                b"{}".to_vec(),
                NO_DEPOSIT,
                MCS_TOKEN_NEW,
            ).then(
            ext_fungible_token::ext(address.parse().unwrap())
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .with_attached_deposit(self.mcs_storage_transfer_in_required)
                .storage_deposit(Some(env::current_account_id()), Some(true))
        )
    }

    /// Checks whether the provided proof is already used
    pub fn is_used_event(&self, order_id: &CryptoHash) -> bool {
        self.used_events.contains(order_id)
    }

    /// Record order id to make sure it is not re-used later for another deposit.
    fn record_order_id(&mut self, order_id: &CryptoHash) -> Balance {
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

    /// Remove order id if transfer in failed.
    fn remove_order_id(&mut self, order_id: &CryptoHash) -> Balance {
        let initial_storage = env::storage_usage();

        if !self.used_events.contains(order_id) {
            return 0;
        }

        self.used_events.remove_raw(order_id);
        let current_storage = env::storage_usage();
        Balance::from(initial_storage - current_storage) * env::storage_byte_cost()
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
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.mcs_tokens.get(&address).is_some(), "token {} is not mcs token", address);

        if let Some(value) = decimals {
            self.token_decimals.insert(&address, &value);
        }

        ext_mcs_token::ext(address.parse().unwrap())
            .with_static_gas(env::prepaid_gas() - OUTER_SET_METADATA_GAS)
            .set_metadata(
                name,
                symbol,
                reference,
                reference_hash,
                decimals,
                icon,
            )
    }

    fn check_amount(&self, token: String, amount: Balance, is_native: bool) {
        let mut decimal = NEAR_DECIMAL;
        if !is_native {
            let decimal_op = self.token_decimals.get(&token);
            assert!(decimal_op.is_some(), "the decimal of token {} is not set", token);
            decimal = decimal_op.unwrap();
        }

        let min_amount = ((10 as u128).pow(decimal as _) as f64) * MIN_TRANSFER_OUT_AMOUNT;
        assert!(amount >= min_amount as Balance, "amount too small, min amount for {} is {}", token, min_amount);
    }

    /// Factory Controller. Controller has extra privileges inside this contract.
    pub fn controller(&self) -> Option<AccountId> {
        env::storage_read(CONTROLLER_STORAGE_KEY)
            .map(|value| String::from_utf8(value).expect("Invalid controller account id").parse().unwrap())
    }

    pub fn set_controller(&mut self, controller: AccountId) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());
        assert!(env::is_valid_account_id(controller.as_bytes()), "invalid account id {}", controller);
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

    /// Return all registered mcs tokens
    pub fn get_mcs_tokens(&self) -> Vec<(String, HashSet<u128>)> {
        self.mcs_tokens.to_vec()
    }

    /// Return all registered fungible tokens (not mcs token)
    pub fn get_fungible_tokens(&self) -> Vec<(String, HashSet<u128>)> {
        self.fungible_tokens.to_vec()
    }

    /// Return all native token to chains
    pub fn get_native_token_to_chains(&self) -> HashSet<u128> {
        self.native_to_chains.clone()
    }

    pub fn add_native_to_chain(&mut self, to_chain: u128) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());

        self.native_to_chains.insert(to_chain);
    }

    pub fn remove_native_to_chain(&mut self, to_chain: u128) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());

        self.native_to_chains.remove(&to_chain);
    }

    pub fn valid_mcs_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.mcs_tokens.get(token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    pub fn add_fungible_token_to_chain(&mut self, token: String, to_chain: u128) -> PromiseOrValue<()> {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.mcs_tokens.get(&token).is_none(), "token name {} exists in mcs token", token);

        if self.fungible_tokens_storage_balance.get(&token).is_none() {
            ext_fungible_token::ext(token.parse().unwrap())
                .with_static_gas(STORAGE_BALANCE_BOUNDS_GAS)
                .storage_balance_bounds()
                .then(Self::ext(env::current_account_id())
                    .with_static_gas(GET_FT_METADATA_GAS + FT_METADATA_GAS
                        + STORAGE_DEPOSIT_FOR_MCS_GAS + STORAGE_DEPOSIT_GAS + FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS)
                    .get_fungible_token_metadata(token, to_chain)).into()
        } else {
            let mut to_chain_set = self.fungible_tokens.get(&token).unwrap_or_default();
            to_chain_set.insert(to_chain);
            self.fungible_tokens.insert(&token, &to_chain_set);
            PromiseOrValue::Value(())
        }
    }

    pub fn get_fungible_token_metadata(&self, token: String, to_chain: u128) -> Promise {
        assert_self();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        let bounds = match env::promise_result(0) {
            PromiseResult::Successful(x) => serde_json::from_slice::<StorageBalanceBounds>(&x).unwrap(),
            _ => panic_str(&*format!("get storage_balance_bounds of token {} failed", token)),
        };

        ext_fungible_token::ext(token.parse().unwrap())
            .with_static_gas(FT_METADATA_GAS)
            .ft_metadata()
            .then(Self::ext(env::current_account_id())
                .with_static_gas(STORAGE_DEPOSIT_FOR_MCS_GAS + STORAGE_DEPOSIT_GAS + FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS)
                .storage_deposit_for_mcs(token, to_chain, bounds.min.0))
    }

    pub fn storage_deposit_for_mcs(&self, token: String, to_chain: u128, min_bound: u128) -> Promise {
        assert_self();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        let metadata = match env::promise_result(0) {
            PromiseResult::Successful(x) => serde_json::from_slice::<FungibleTokenMetadata>(&x).unwrap(),
            _ => panic_str(&*format!("get storage_balance_bounds of token {} failed", token)),
        };

        ext_fungible_token::ext(token.parse().unwrap())
            .with_static_gas(STORAGE_DEPOSIT_GAS)
            .with_attached_deposit(min_bound)
            .storage_deposit(Some(env::current_account_id()), Some(true))
            .then(Self::ext(env::current_account_id())
                .with_static_gas(FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS)
                .finish_add_fungible_token_to_chain(token, to_chain, min_bound, metadata.decimals))
    }

    pub fn finish_add_fungible_token_to_chain(&mut self, token: String, to_chain: u128, min_bound: u128, decimals: u8) {
        assert_self();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        let _balance = match env::promise_result(0) {
            PromiseResult::Successful(x) => serde_json::from_slice::<StorageBalance>(&x).unwrap(),
            _ => panic_str(&*format!("storage deposit to token {} for mcs failed", token)),
        };

        let mut to_chain_set = self.fungible_tokens.get(&token).unwrap_or(Default::default());
        to_chain_set.insert(to_chain);
        self.fungible_tokens.insert(&token, &to_chain_set);
        self.fungible_tokens_storage_balance.insert(&token, &min_bound);
        self.token_decimals.insert(&token, &decimals);
    }

    pub fn remove_fungible_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());

        let mut to_chain_set = self.fungible_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain);
        if to_chain_set.is_empty() {
            self.fungible_tokens.remove(&token);
        } else {
            self.fungible_tokens.insert(&token, &to_chain_set);
        }
    }

    pub fn valid_fungible_token_out(&self, token: &String, to_chain: u128) -> bool {
        let to_chain_set_wrap = self.fungible_tokens.get(token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain)
    }

    pub fn add_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.insert(to_chain);
        self.mcs_tokens.insert(&token, &to_chain_set);
    }

    pub fn remove_mcs_token_to_chain(&mut self, token: String, to_chain: u128) {
        assert!(self.controller_or_self(), "unexpected caller {}", env::predecessor_account_id());

        let mut to_chain_set = self.mcs_tokens.get(&token).expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain);
        self.mcs_tokens.insert(&token, &to_chain_set);
    }

    fn get_order_id(&mut self, token: &String, from: &String, to: &Vec<u8>, amount: u128, to_chain_id: u128) -> CryptoHash {
        let mut data = self.nonce.try_to_vec().unwrap();
        data.extend(from.as_bytes());
        data.extend(to);
        data.extend(token.as_bytes());
        data.extend(amount.try_to_vec().unwrap());
        data.extend(self.near_chain_id.try_to_vec().unwrap());
        data.extend(to_chain_id.try_to_vec().unwrap());
        self.nonce = self.nonce + 1;
        CryptoHash::try_from(env::sha256(&data[..])).unwrap()
    }

    fn is_native_token(&self, token: Vec<u8>) -> bool {
        let addr = self.native_token_address().0;
        token.eq(&addr)
    }

    fn native_token_address(&self) -> (Vec<u8>, String) {
        let addr: Vec<u8> = vec![0; 20];
        (addr.clone(), String::from_utf8(addr).unwrap())
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for MapCrossChainService {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id().to_string();
        let transfer_msg: FungibleTokenMsg = serde_json::from_str(&msg).unwrap();

        let from = sender_id.to_string();
        if transfer_msg.msg_type == 0 {
            self.check_not_paused(PAUSE_TRANSFER_OUT_TOKEN);
            assert!(self.valid_fungible_token_out(&token, transfer_msg.to_chain),
                    "transfer token {} to chain {} is not supported", token, transfer_msg.to_chain);
            self.check_amount(token.clone(), amount.0, false);

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
        } else if transfer_msg.msg_type == 1 {
            self.check_not_paused(PAUSE_DEPOSIT_OUT_TOKEN);
            assert!(self.valid_fungible_token_out(&token, MAP_CHAIN_ID)
                        || self.valid_mcs_token_out(&token, MAP_CHAIN_ID),
                    "deposit token {} to chain {} is not supported", token, MAP_CHAIN_ID);
            self.check_amount(token.clone(), amount.0, false);

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
            env::panic_str(format!("transfer msg typ {} is not supported", transfer_msg.msg_type).as_ref());
        }

        PromiseOrValue::Value(U128::from(0))
    }
}

admin_controlled::impl_admin_controlled!(MapCrossChainService, paused);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils::VMContextBuilder, testing_env, env::sha256};
    use std::convert::TryInto;
    use uint::rustc_hex::ToHex;
    use map_light_client::header::Header;
    use map_light_client::proof::Receipt;
    use map_light_client::G2;

    const UNPAUSE_ALL: Mask = 0;
    const NEAR_CHAIN_ID: u128 = 1313161555;
    const ETH_CHAIN_ID: u128 = 1;
    const STORAGE_BALANCE: u128 = 10000000;

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

    fn alice() -> (AccountId, Vec<u8>) {
        ("alice.near".parse().unwrap(), hex::decode("ab175474e89094c44da98b954eedeac495271d0f").unwrap())
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
        "wrap.near".to_string()
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

    fn mcs_contract() -> MapCrossChainService {
        MapCrossChainService {
            map_client_account: prover().parse().unwrap(),
            map_bridge_address: validate_eth_address(map_bridge_address()),
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            fungible_tokens_storage_balance: UnorderedMap::new(b"s".to_vec()),
            token_decimals: UnorderedMap::new(b"d".to_vec()),
            native_to_chains: Default::default(),
            used_events: UnorderedSet::new(b"u".to_vec()),
            owner_pk: env::signer_account_pk(),
            mcs_storage_transfer_in_required: STORAGE_BALANCE,
            wrapped_token: wrap_token(),
            near_chain_id: NEAR_CHAIN_ID,  // 1313161555 for testnet
            nonce: 0,
            paused: Mask::default(),
        }
    }

    #[test]
    #[should_panic]
    fn test_fail_deploy_mcs_token() {
        let mut contract = mcs_contract();
        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE,
        );
        contract.deploy_mcs_token(map_bridge_address());
    }

    #[test]
    #[should_panic]
    fn test_fail_transfer_in_no_event() {
        let mut contract = mcs_contract();
        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: env::storage_byte_cost() * 1000
        );
        contract.transfer_in(sample_proof(), 0);
    }

    #[test]
    fn test_deploy_mcs_token() {
        let mut contract = mcs_contract();
        set_env!(
            current_account_id: map_cross_chain_service(),
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE * 2,
        );

        contract.deploy_mcs_token(map_bridge_address());
        let account = format!("{}.{}", map_bridge_address(), map_cross_chain_service());
        assert!(contract.mcs_tokens.get(&account).is_some());

        let uppercase_address = "0f5Ea0A652E851678Ebf77B69484bFcD31F9459B".to_string();
        contract.deploy_mcs_token(uppercase_address.clone());
        let account = format!("{}.{}", uppercase_address.to_lowercase(), map_cross_chain_service());
        assert!(contract.mcs_tokens.get(&account).is_some());
    }

    #[test]
    #[should_panic]
    fn test_transfer_out_token_no_tochian() {
        let token = mcs_token().0;
        let from = alice().0;
        let to = alice().1;
        let mut contract = mcs_contract();

        set_env!(
            predecessor_account_id: alice().0,
            attached_deposit: MCS_TOKEN_INIT_BALANCE * 2
        );
        contract.deploy_mcs_token(mcs_token().0);

        set_env!(
            current_account_id: map_cross_chain_service(),
            predecessor_account_id: format!("{}.{}", token, map_cross_chain_service())
        );

        contract.transfer_out_token(token, to, U128(1_000), ETH_CHAIN_ID);
    }
}
