extern crate core;

use crate::management::{ChainType, ChainType::*};
use crate::swap::{Action, CoreSwapMessage, SwapAction};
use crate::token_receiver::SwapInfo;
use crate::traits::*;
use admin_controlled::{AdminControlled, Mask};
use event::*;
use map_light_client::proof::ReceiptProof;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::env::panic_str;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, log, near_bindgen, AccountId, Balance, CryptoHash, Gas, PanicOnDefault,
    Promise, PromiseOrValue, PromiseResult,
};
use prover::*;
use std::collections::{HashMap, HashSet};

mod bytes;
mod event;
mod management;
pub mod prover;
mod swap;
pub mod token_receiver;
mod tokens;
mod traits;

const NO_DEPOSIT: Balance = 0;

/// Gas to call ft_transfer on ext fungible contract
const FT_TRANSFER_GAS: Gas = Gas(5_000_000_000_000);
/// Gas to call callback_post_swap_in on core contract
const CALLBACK_POST_SWAP_IN_GAS: Gas = Gas(22_000_000_000_000);

/// Gas to call storage_deposit on ext fungible contract
const STORAGE_DEPOSIT_GAS: Gas = Gas(5_000_000_000_000);

/// Gas to call finish_init on mcs contract
const FINISH_INIT_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call mint/burn method on mcs token.
const MINT_GAS: Gas = Gas(5_000_000_000_000);
const BURN_GAS: Gas = Gas(5_000_000_000_000);

/// Gas to call near_withdraw and near_deposit on wrap near contract
const NEAR_WITHDRAW_GAS: Gas = Gas(5_000_000_000_000);
const NEAR_DEPOSIT_GAS: Gas = Gas(7_000_000_000_000);

/// Gas to call finish_verify_proof method.
const TRANSFER_IN_SINGLE_EVENT_GAS: Gas = Gas(60_000_000_000_000);

/// Gas to call transfer_in_native_token method.
const TRANSFER_IN_NATIVE_TOKEN_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish_transfer_in method.
const FINISH_TRANSFER_IN_GAS: Gas = Gas(30_000_000_000_000);

/// Gas to call finish_token_out method.
const FINISH_TOKEN_OUT_GAS: Gas = Gas(5_000_000_000_000);

/// Gas to call report_failure method.
const REPORT_FAILURE_GAS: Gas = Gas(5_000_000_000_000);

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = Gas(83_000_000_000_000);
/// Gas to call process_swap_in method.
const PROCESS_SWAP_IN_GAS: Gas = Gas(200_000_000_000_000);
/// Gas to call process_swap_out method.
const PROCESS_SWAP_OUT_GAS: Gas = Gas(250_000_000_000_000);
/// Gas to call callback_process_native_swap method.
const CALLBACK_PROCESS_NATIVE_SWAP_GAS: Gas = Gas(25_000_000_000_000);

const MIN_TRANSFER_OUT_AMOUNT: f64 = 0.001;
const NEAR_DECIMAL: u8 = 24;

const PAUSE_DEPLOY_TOKEN: Mask = 1 << 0;
const PAUSE_TRANSFER_IN: Mask = 1 << 1;
const PAUSE_TRANSFER_OUT_TOKEN: Mask = 1 << 2;
const PAUSE_TRANSFER_OUT_NATIVE: Mask = 1 << 3;
const PAUSE_DEPOSIT_OUT_TOKEN: Mask = 1 << 4;
const PAUSE_DEPOSIT_OUT_NATIVE: Mask = 1 << 5;
const PAUSE_SWAP_IN: Mask = 1 << 6;
const PAUSE_SWAP_OUT_TOKEN: Mask = 1 << 7;
const PAUSE_SWAP_OUT_NATIVE: Mask = 1 << 8;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MAPOServiceV2 {
    /// The account of the map light client that we can use to prove
    pub map_client_account: AccountId,
    /// Address of the MAP bridge contract.
    pub map_bridge_address: Address,
    /// Set of created MCSToken contracts.
    pub mcs_tokens: UnorderedMap<AccountId, HashSet<u128>>,
    /// Set of other fungible token contracts.
    pub fungible_tokens: UnorderedMap<AccountId, HashSet<u128>>,
    /// Map of other fungible token contracts and their min storage balance.
    pub fungible_tokens_storage_balance: UnorderedMap<AccountId, u128>,
    /// Map of token contracts and their decimals
    pub token_decimals: UnorderedMap<AccountId, u8>,
    /// Set of other fungible token contracts.
    pub native_to_chains: HashSet<u128>,
    /// Map of chain id and chain type
    pub chain_id_type_map: UnorderedMap<u128, ChainType>,
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<CryptoHash>,
    /// Account of the owner
    pub owner: AccountId,
    /// Balance required to register a new account in the MCSToken
    pub mcs_storage_balance_min: Balance,
    // Wrap token for near
    pub wrapped_token: AccountId,
    // Near chain id
    pub near_chain_id: u128,
    // MAP chain id
    pub map_chain_id: u128,
    // Nonce to generate order id
    pub nonce: u128,
    /// Mask determining all paused functions
    pub paused: Mask,

    pub registered_tokens: UnorderedMap<AccountId, bool>,

    /// SWAP related
    pub ref_exchange: AccountId,
    pub core_idle: Vec<AccountId>,
    pub core_total: Vec<AccountId>,
    pub amount_out: HashMap<AccountId, U128>,
    pub lost_found: UnorderedMap<AccountId, HashMap<AccountId, Balance>>,
}

#[near_bindgen]
impl MAPOServiceV2 {
    /// Initializes the contract.
    /// `map_client_account`: NEAR account of the MAP light client contract;
    /// `map_bridge_address`: the address of the MCS contract on MAP blockchain, in hex.
    /// `wrapped_token`: the wrap near contract account id
    /// `near_chain_id`: the chain id of the near blockchain
    pub fn init(
        owner: AccountId,
        map_light_client: AccountId,
        map_bridge_address: String,
        wrapped_token: AccountId,
        near_chain_id: U128,
        map_chain_id: U128,
        ref_exchange: AccountId,
        butter_core: Vec<AccountId>,
    ) -> Promise {
        assert!(!env::state_exists(), "Already initialized");
        let map_bridge_address = validate_eth_address(map_bridge_address);

        let storage_balance =
            near_contract_standards::fungible_token::FungibleToken::new(b"t".to_vec())
                .account_storage_usage as Balance
                * env::storage_byte_cost();

        let mut promise = ext_fungible_token::ext(wrapped_token.clone())
            .with_static_gas(STORAGE_DEPOSIT_GAS)
            .with_attached_deposit(storage_balance)
            .storage_deposit(Some(env::current_account_id()), Some(true))
            .then(
                ext_fungible_token::ext(wrapped_token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(storage_balance)
                    .storage_deposit(Some(ref_exchange.clone()), Some(true)),
            );
        for core in butter_core.clone() {
            promise = promise.then(
                ext_fungible_token::ext(wrapped_token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(storage_balance)
                    .storage_deposit(Some(core), Some(true)),
            );
        }
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(FINISH_INIT_GAS)
                .finish_init(
                    owner,
                    map_light_client,
                    map_bridge_address,
                    wrapped_token,
                    near_chain_id,
                    map_chain_id,
                    storage_balance.into(),
                    ref_exchange,
                    butter_core,
                ),
        )
    }
    #[init]
    #[private]
    pub fn finish_init(
        owner: AccountId,
        map_light_client: AccountId,
        map_bridge_address: Address,
        wrapped_token: AccountId,
        near_chain_id: U128,
        map_chain_id: U128,
        storage_balance: U128,
        ref_exchange: AccountId,
        butter_core: Vec<AccountId>,
    ) -> Self {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        let _balance = match env::promise_result(0) {
            PromiseResult::Successful(x) => serde_json::from_slice::<StorageBalance>(&x).unwrap(),
            _ => panic_str("wnear contract storage deposit failed"),
        };

        Self {
            map_client_account: map_light_client,
            map_bridge_address,
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            fungible_tokens_storage_balance: UnorderedMap::new(b"s".to_vec()),
            token_decimals: UnorderedMap::new(b"d".to_vec()),
            native_to_chains: Default::default(),
            chain_id_type_map: UnorderedMap::new(b"c".to_vec()),
            used_events: UnorderedSet::new(b"u".to_vec()),
            owner,
            mcs_storage_balance_min: storage_balance.into(),
            wrapped_token,
            near_chain_id: near_chain_id.into(), // 1313161555 for testnet
            map_chain_id: map_chain_id.into(),
            nonce: 0,
            paused: Mask::default(),
            registered_tokens: UnorderedMap::new(b"r".to_vec()),
            ref_exchange,
            core_idle: butter_core.clone(),
            core_total: butter_core,
            amount_out: Default::default(),
            lost_found: UnorderedMap::new(b"l".to_vec()),
        }
    }

    /// Transfer from Map to NEAR based on the proof of the locked tokens or messages.
    /// Must attach enough NEAR funds to cover for storage of the proof.
    #[payable]
    pub fn transfer_in(&mut self, receipt_proof: ReceiptProof, index: usize) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_IN);

        let logs = &receipt_proof.receipt.logs;
        assert!(index < logs.len(), "index exceeds event size");

        let (map_bridge_address, event) =
            TransferOutEvent::from_log_entry_data(logs.get(index).unwrap())
                .unwrap_or_else(|| panic_str("not map transfer out event"));
        assert_eq!(
            self.map_bridge_address,
            map_bridge_address,
            "unexpected map mcs address: {}",
            hex::encode(map_bridge_address)
        );
        self.check_map_transfer_out_event(&event);

        log!(
            "get transfer in event: {}",
            serde_json::to_string(&event).unwrap()
        );

        ext_map_light_client::ext(self.map_client_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_proof_data(receipt_proof)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(TRANSFER_IN_SINGLE_EVENT_GAS + FINISH_TRANSFER_IN_GAS)
                    .with_attached_deposit(env::attached_deposit())
                    .callback_process_transfer_in(&event),
            )
    }

    #[payable]
    pub fn swap_in(&mut self, receipt_proof: ReceiptProof, index: usize) -> Promise {
        self.check_not_paused(PAUSE_SWAP_IN);

        let logs = &receipt_proof.receipt.logs;
        assert!(index < logs.len(), "index exceeds event size");

        let (map_bridge_address, event) =
            SwapOutEvent::from_log_entry_data(logs.get(index).unwrap())
                .unwrap_or_else(|| panic_str("not map swap out event"));
        assert_eq!(
            self.map_bridge_address,
            map_bridge_address,
            "unexpected map mcs address: {}",
            hex::encode(map_bridge_address)
        );
        self.check_map_swap_out_event(&event);

        log!(
            "get swap in event: {}",
            serde_json::to_string(&event).unwrap()
        );

        ext_map_light_client::ext(self.map_client_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_proof_data(receipt_proof)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(PROCESS_SWAP_IN_GAS)
                    .with_attached_deposit(env::attached_deposit())
                    .callback_process_swap_in(&event),
            )
    }

    #[payable]
    pub fn transfer_out_token(
        &mut self,
        token: AccountId,
        to: Vec<u8>,
        amount: U128,
        to_chain: U128,
    ) -> Promise {
        assert_one_yocto();
        self.check_not_paused(PAUSE_TRANSFER_OUT_TOKEN);
        self.mcs_token_out(token, to, amount, to_chain, MsgType::Transfer)
    }

    #[payable]
    pub fn deposit_out_token(&mut self, token: AccountId, to: Vec<u8>, amount: U128) -> Promise {
        assert_one_yocto();
        self.check_not_paused(PAUSE_DEPOSIT_OUT_TOKEN);
        self.mcs_token_out(
            token,
            to,
            amount,
            self.map_chain_id.into(),
            MsgType::Deposit,
        )
    }

    pub fn swap_out_token(
        &mut self,
        to_chain: U128,
        token: AccountId,
        from: AccountId,
        to: Vec<u8>,
        amount: U128,
        swap_data: SwapData,
    ) -> PromiseOrValue<U128> {
        self.check_token_to_chain(&token, to_chain);
        self.check_to_account(to.clone(), to_chain.into());
        self.check_amount(&token, amount.0);
        let order_id = self.get_order_id(&from.to_string(), &to, to_chain.into());

        let event = SwapOutEvent {
            from_chain: self.near_chain_id.into(),
            to_chain,
            order_id,
            token: Vec::from(token.as_bytes()),
            from: Vec::from(from.as_bytes()),
            to,
            amount,
            swap_data: swap_data.abi_encode(),
            raw_swap_data: swap_data,
        };

        if self.valid_mcs_token_out(&token, to_chain) {
            ext_mcs_token::ext(token)
                .with_static_gas(BURN_GAS)
                .burn(env::current_account_id(), amount)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TOKEN_OUT_GAS)
                        .finish_token_out(MCSEvent::Swap(event)),
                )
                .into()
        } else {
            event.emit();

            PromiseOrValue::Value(U128(0))
        }
    }

    #[payable]
    pub fn transfer_out_native(&mut self, to: Vec<u8>, to_chain: U128) -> Promise {
        self.check_not_paused(PAUSE_TRANSFER_OUT_NATIVE);
        self.native_token_out(to, env::attached_deposit(), to_chain, MsgType::Transfer)
    }

    #[payable]
    pub fn deposit_out_native(&mut self, to: Vec<u8>) -> Promise {
        self.check_not_paused(PAUSE_DEPOSIT_OUT_NATIVE);
        self.native_token_out(
            to,
            env::attached_deposit(),
            self.map_chain_id.into(),
            MsgType::Deposit,
        )
    }

    #[payable]
    pub fn swap_out_native(&mut self, to: Vec<u8>, to_chain: U128, swap_info: SwapInfo) -> Promise {
        self.check_not_paused(PAUSE_SWAP_OUT_NATIVE);
        self.check_to_account(to.clone(), to_chain.into());

        let amount = env::attached_deposit();
        self.check_amount(&self.wrapped_token, amount);

        ext_wnear_token::ext(self.wrapped_token.clone())
            .with_static_gas(NEAR_DEPOSIT_GAS)
            .with_attached_deposit(amount)
            .near_deposit()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(PROCESS_SWAP_OUT_GAS)
                    .process_token_swap_out(
                        to_chain,
                        self.native_token_address().1.parse().unwrap(),
                        env::signer_account_id(),
                        to,
                        amount.into(),
                        swap_info,
                    ),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(CALLBACK_PROCESS_NATIVE_SWAP_GAS)
                    .callback_process_native_swap(amount),
            )
    }

    #[private]
    pub fn callback_process_native_swap(&mut self, amount: Balance) -> PromiseOrValue<()> {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        let refund_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Failed => amount,
            PromiseResult::Successful(x) => {
                let refund_amount = serde_json::from_slice::<U128>(&x).unwrap();
                refund_amount.0
            }
        };
        if refund_amount == 0 {
            return PromiseOrValue::Value(());
        }

        ext_wnear_token::ext(self.wrapped_token.clone())
            .with_static_gas(NEAR_WITHDRAW_GAS)
            .with_attached_deposit(1)
            .near_withdraw(refund_amount.into())
            .then(Promise::new(env::signer_account_id()).transfer(refund_amount))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(REPORT_FAILURE_GAS)
                    .report_failure("swap failed, refund amount to signer".to_string()),
            )
            .into()
    }

    #[payable]
    #[private]
    pub fn callback_process_transfer_in(&mut self, event: &TransferOutEvent) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Failed => self.revert_state(
                env::attached_deposit(),
                None,
                "verify proof failed".to_string(),
            ),
            PromiseResult::Successful(_) => self.process_transfer_in(event),
        }
    }

    #[payable]
    #[private]
    pub fn callback_process_swap_in(&mut self, event: &SwapOutEvent) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Failed => self.revert_state(
                env::attached_deposit(),
                None,
                "verify proof failed".to_string(),
            ),
            PromiseResult::Successful(_) => {
                if let Some(transfer_out_event) = event.to_transfer_out_event() {
                    self.process_transfer_in(&transfer_out_event)
                } else {
                    self.process_swap_in(event)
                }
            }
        }
    }

    fn process_transfer_in(&mut self, event: &TransferOutEvent) -> Promise {
        let mut ret_deposit = env::attached_deposit();
        if self.is_used_event(&event.order_id) {
            let err_msg = "the transfer in event is already processed".to_string();
            return self.revert_state(ret_deposit, None, err_msg);
        }

        let required_deposit = self.record_order_id(&event.order_id);
        if ret_deposit < required_deposit {
            let err_msg = format!(
                "not enough deposit for record proof, exp: {}, cur: {}",
                required_deposit, ret_deposit
            );
            self.remove_order_id(&event.order_id);
            return self.revert_state(ret_deposit, None, err_msg);
        }

        let to_chain_token = event.get_to_chain_token();
        let to = event.get_to_account();
        env::log_str(
            format!(
                "start to transfer in token: {:?}, to: {:?}, amount: {}",
                to_chain_token, to, event.amount.0
            )
            .as_str(),
        );

        ret_deposit -= required_deposit;
        if self.is_native_token(&to_chain_token) {
            ext_wnear_token::ext(self.wrapped_token.clone())
                .with_static_gas(NEAR_WITHDRAW_GAS)
                .with_attached_deposit(1)
                .near_withdraw(event.amount)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(TRANSFER_IN_NATIVE_TOKEN_GAS)
                        .with_attached_deposit(ret_deposit)
                        .transfer_in_native_token(event),
                )
        } else if self.mcs_tokens.get(&to_chain_token).is_some() {
            if ret_deposit < self.mcs_storage_balance_min {
                let err_msg = format!(
                    "not enough deposit for mcs token mint, exp: {}, cur: {}",
                    self.mcs_storage_balance_min, ret_deposit
                );
                return self.revert_state(ret_deposit, Some(event.order_id), err_msg);
            }
            ret_deposit -= self.mcs_storage_balance_min;

            ext_mcs_token::ext(to_chain_token)
                .with_static_gas(MINT_GAS)
                .with_attached_deposit(self.mcs_storage_balance_min)
                .mint(to, event.amount)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TRANSFER_IN_GAS)
                        .with_attached_deposit(ret_deposit)
                        .finish_transfer_in(event),
                )
        } else if self.fungible_tokens.get(&to_chain_token).is_some() {
            // to_chain_token is fungible token
            let min_storage_balance = self
                .fungible_tokens_storage_balance
                .get(&to_chain_token)
                .unwrap();
            if ret_deposit < min_storage_balance {
                let err_msg = format!(
                    "not enough deposit for ft transfer, exp: {}, cur: {}",
                    min_storage_balance, ret_deposit
                );
                return self.revert_state(ret_deposit, Some(event.order_id), err_msg);
            }
            ret_deposit -= min_storage_balance;

            ext_fungible_token::ext(to_chain_token.clone())
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .with_attached_deposit(min_storage_balance)
                .storage_deposit(Some(to.clone()), Some(true))
                .then(
                    ext_fungible_token::ext(to_chain_token)
                        .with_static_gas(FT_TRANSFER_GAS)
                        .with_attached_deposit(1)
                        .ft_transfer(to, event.amount, None),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TRANSFER_IN_GAS)
                        .with_attached_deposit(ret_deposit)
                        .finish_transfer_in(event),
                )
        } else {
            panic_str(
                format!(
                    "to_chain_token {} is not mcs token or fungible token or native token",
                    to_chain_token
                )
                .as_str(),
            )
        }
    }

    fn process_swap_in(&mut self, event: &SwapOutEvent) -> Promise {
        let mut ret_deposit = env::attached_deposit();
        let core_opt = self.core_idle.pop();
        if core_opt.is_none() {
            return self.revert_state(ret_deposit, None, "no idle core!".to_string());
        }
        let core = core_opt.unwrap();

        if self.is_used_event(&event.order_id) {
            let err_msg = "the swap in event is already processed".to_string();
            return self.revert_state(ret_deposit, None, err_msg);
        }

        let required_deposit = self.record_order_id(&event.order_id);
        if ret_deposit < required_deposit {
            let err_msg = format!(
                "not enough deposit for record proof, exp: {}, cur: {}",
                required_deposit, ret_deposit
            );
            self.remove_order_id(&event.order_id);
            return self.revert_state(ret_deposit, None, err_msg);
        }
        ret_deposit -= required_deposit;

        let token_in = event.get_token_in();
        let token_out = event.get_token_out();
        let target_account = event.get_to_account();
        let storage_balance_in = self.get_storage_deposit_balance(&token_in, true);
        let storage_balance_out = self.get_storage_deposit_balance(&token_out, false);
        if ret_deposit < storage_balance_in + storage_balance_out {
            let err_msg = format!(
                "not enough deposit for storage deposit, exp: {}, cur: {}",
                storage_balance_in + storage_balance_out,
                ret_deposit
            );
            return self.revert_state(ret_deposit, Some(event.order_id), err_msg);
        }
        // make sure ret_deposit is enough for token in storage deposit
        ret_deposit -= storage_balance_out;
        if storage_balance_out > 0 {
            ext_fungible_token::ext(token_out.clone())
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .with_attached_deposit(storage_balance_out)
                .storage_deposit(Some(target_account.clone()), Some(true));
        }

        let actions = event
            .raw_swap_data
            .swap_param
            .clone()
            .into_iter()
            .map(|param| Action::Swap(param.to_swap_action()))
            .collect();

        let core_swap_msg = CoreSwapMessage {
            actions,
            target_account: target_account.clone(),
            target_token: Some(token_out),
        };

        self.call_core_swap_in_directly(
            core,
            token_in,
            target_account,
            event.amount,
            core_swap_msg,
            ret_deposit,
            event.order_id,
            storage_balance_in,
        )
    }

    fn revert_state(
        &mut self,
        mut ret_deposit: Balance,
        order_id: Option<CryptoHash>,
        err_msg: String,
    ) -> Promise {
        if let Some(id) = order_id {
            ret_deposit += self.remove_order_id(&id);
        }
        Promise::new(env::signer_account_id())
            .transfer(ret_deposit)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(REPORT_FAILURE_GAS)
                    .report_failure(err_msg),
            )
    }

    #[payable]
    #[private]
    pub fn transfer_in_native_token(&mut self, event: &TransferOutEvent) -> Promise {
        assert_eq!(1, env::promise_results_count(), "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => {
                let to = String::from_utf8(event.to.clone()).unwrap();
                Promise::new(to.parse().unwrap())
                    .transfer(event.amount.into())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(FINISH_TRANSFER_IN_GAS)
                            .with_attached_deposit(env::attached_deposit())
                            .finish_transfer_in(event),
                    )
            }
            _ => self.revert_state(
                env::attached_deposit(),
                Some(event.order_id),
                "near withdraw failed".to_string(),
            ),
        }
    }

    #[payable]
    #[private]
    pub fn finish_transfer_in(&mut self, event: &TransferOutEvent) -> Promise {
        assert_eq!(1, env::promise_results_count(), "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => {
                Promise::new(env::signer_account_id()).transfer(env::attached_deposit())
            }
            _ => {
                let mut err_msg = "transfer in token failed".to_string();
                let mut promise = Promise::new(env::signer_account_id());
                if self.is_native_token(&event.get_to_chain_token()) {
                    promise = promise.transfer(env::attached_deposit()).then(
                        ext_wnear_token::ext(self.wrapped_token.clone())
                            .with_static_gas(NEAR_DEPOSIT_GAS)
                            .with_attached_deposit(event.amount.into())
                            .near_deposit(),
                    );
                    err_msg =
                        "transfer in token failed, maybe TO account does not exist".to_string()
                } else {
                    promise = promise
                        .transfer(env::attached_deposit() + self.remove_order_id(&event.order_id))
                }
                promise.then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(REPORT_FAILURE_GAS)
                        .report_failure(err_msg),
                )
            }
        }
    }

    #[private]
    pub fn report_failure(err: String) {
        panic_str(err.as_str())
    }

    /// Finish transfer out once the nep141 token is burned from MCSToken contract or native token is transferred.
    #[private]
    pub fn finish_token_out(&self, mcs_event: MCSEvent) -> U128 {
        assert_eq!(1, env::promise_results_count(), "ERR_TOO_MANY_RESULTS");
        assert_eq!(
            PromiseResult::Successful(vec![]),
            env::promise_result(0),
            "burn mcs token or call near_deposit() failed"
        );

        mcs_event.emit();
        U128(0)
    }

    /// Checks whether the provided proof is already used
    pub fn is_used_event(&self, order_id: &CryptoHash) -> bool {
        self.used_events.contains(order_id)
    }
}

impl MAPOServiceV2 {
    fn mcs_token_out(
        &mut self,
        token: AccountId,
        to: Vec<u8>,
        amount: U128,
        to_chain: U128,
        msg_type: MsgType,
    ) -> Promise {
        if self.valid_mcs_token_out(&token, to_chain) {
            assert!(
                msg_type == MsgType::Transfer || msg_type == MsgType::Deposit,
                "msg type is invalid: {:?}",
                msg_type
            );
            self.check_to_account(to.clone(), to_chain.into());
            self.check_amount(&token, amount.0);
            let from = env::signer_account_id().to_string();
            let order_id = self.get_order_id(&from, &to, to_chain.into());

            let event = if msg_type == MsgType::Transfer {
                MCSEvent::Transfer(TransferOutEvent {
                    from_chain: self.near_chain_id.into(),
                    to_chain,
                    from: from.clone().into_bytes(),
                    to,
                    order_id,
                    token: token.as_bytes().to_vec(),
                    to_chain_token: "".to_string().into_bytes(),
                    amount,
                })
            } else {
                MCSEvent::Deposit(DepositOutEvent {
                    from_chain: self.near_chain_id.into(),
                    to_chain,
                    from: from.clone(),
                    to,
                    order_id,
                    token: token.to_string(),
                    amount,
                })
            };

            ext_mcs_token::ext(token)
                .with_static_gas(BURN_GAS)
                .burn(from.parse().unwrap(), amount)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(FINISH_TOKEN_OUT_GAS)
                        .finish_token_out(event),
                )
        } else if self.valid_fungible_token_out(&token, to_chain) {
            env::panic_str(
                format!(
                    "non mcs fungible token {} should called from fungible token directly",
                    token
                )
                .as_ref(),
            );
        } else {
            env::panic_str(
                format!("token {} to chain {} is not supported", token, to_chain.0).as_ref(),
            );
        }
    }

    fn native_token_out(
        &mut self,
        to: Vec<u8>,
        amount: Balance,
        to_chain: U128,
        msg_type: MsgType,
    ) -> Promise {
        assert!(
            msg_type == MsgType::Transfer || msg_type == MsgType::Deposit,
            "msg type is invalid: {:?}",
            msg_type
        );
        self.check_to_account(to.clone(), to_chain.into());
        assert!(amount > 0, "amount should > 0");
        assert!(
            self.native_to_chains.contains(&to_chain.into()),
            "native token to chain {} is not supported",
            to_chain.0
        );
        self.check_amount(&self.wrapped_token, amount);

        let from = env::signer_account_id().to_string();
        let order_id = self.get_order_id(&from, &to, to_chain.into());

        let event = if msg_type == MsgType::Transfer {
            MCSEvent::Transfer(TransferOutEvent {
                from_chain: self.near_chain_id.into(),
                to_chain,
                from: from.into_bytes(),
                to,
                order_id,
                token: self.native_token_address().0,
                to_chain_token: "".to_string().into_bytes(),
                amount: amount.into(),
            })
        } else {
            MCSEvent::Deposit(DepositOutEvent {
                from_chain: self.near_chain_id.into(),
                to_chain,
                from: from.clone(),
                to,
                order_id,
                token: self.native_token_address().1,
                amount: amount.into(),
            })
        };

        ext_wnear_token::ext(self.wrapped_token.clone())
            .with_static_gas(NEAR_DEPOSIT_GAS)
            .with_attached_deposit(amount)
            .near_deposit()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_TOKEN_OUT_GAS)
                    .finish_token_out(event),
            )
    }

    fn get_storage_deposit_balance(&self, token: &AccountId, deposit_wrap_token: bool) -> Balance {
        if self.mcs_tokens.get(token).is_some() {
            self.mcs_storage_balance_min
        } else if self.fungible_tokens.get(token).is_some() {
            self.fungible_tokens_storage_balance.get(token).unwrap()
        } else if deposit_wrap_token && self.is_native_token(token) {
            self.mcs_storage_balance_min
        } else {
            0
        }
    }

    fn check_map_transfer_out_event(&self, event: &TransferOutEvent) {
        assert_eq!(
            self.near_chain_id, event.to_chain.0,
            "unexpected to chain: {}",
            event.to_chain.0
        );
        assert!(
            !self.is_used_event(&event.order_id),
            "the event with order id {} is used",
            hex::encode(event.order_id)
        );

        event.basic_check();

        self.check_token(&event.get_to_chain_token());
    }

    fn check_map_swap_out_event(&self, event: &SwapOutEvent) {
        assert_eq!(
            self.near_chain_id, event.to_chain.0,
            "unexpected to chain: {}",
            event.to_chain.0
        );
        assert!(
            !self.is_used_event(&event.order_id),
            "the event with order id {} is used",
            hex::encode(event.order_id)
        );

        event.basic_check();

        self.check_token(&event.get_token_in());
        self.check_token(&event.get_token_out());
    }

    fn is_owner(&self) -> bool {
        env::predecessor_account_id() == self.owner
    }

    fn check_amount(&self, token: &AccountId, amount: Balance) {
        let mut decimal = NEAR_DECIMAL;
        if !self.is_native_token(token) {
            let decimal_op = self.token_decimals.get(token);
            assert!(
                decimal_op.is_some(),
                "the decimal of token {} is not set",
                token
            );
            decimal = decimal_op.unwrap();
        }

        let min_amount = (10_u128.pow(decimal as _) as f64) * MIN_TRANSFER_OUT_AMOUNT;
        assert!(
            amount >= min_amount as Balance,
            "amount too small, min amount for {} is {}",
            token,
            min_amount
        );
    }

    fn check_token(&self, token: &AccountId) {
        assert!(
            self.mcs_tokens.get(token).is_some()
                || self.fungible_tokens.get(token).is_some()
                || self.is_native_token(token),
            "to_chain_token {} is not mcs token or fungible token or native token",
            token
        );
    }

    fn check_token_to_chain(&self, token: &AccountId, to_chain: U128) {
        if self.is_native_token(token) {
            assert!(
                self.get_native_token_to_chains().contains(&to_chain),
                "token {:?} to chain {} is not supported",
                token,
                to_chain.0
            );
        } else {
            assert!(
                self.valid_mcs_token_out(token, to_chain)
                    || self.valid_fungible_token_out(token, to_chain),
                "token {:?} to chain {} is not supported",
                token,
                to_chain.0
            );
        }
    }

    /*
    function _getOrderId(address _from, bytes memory _to, uint256 _toChain) internal returns (bytes32){
        return keccak256(abi.encodePacked(address(this), nonce++, selfChainId, _toChain, _from, _to));
    }
     */
    fn get_order_id(&mut self, from: &String, to: &Vec<u8>, to_chain_id: u128) -> CryptoHash {
        let mut data: Vec<u8> = Vec::new();
        data.extend(env::current_account_id().as_bytes());
        data.extend(self.nonce.try_to_vec().unwrap());
        data.extend(self.near_chain_id.try_to_vec().unwrap());
        data.extend(to_chain_id.try_to_vec().unwrap());
        data.extend(from.as_bytes());
        data.extend(to);
        self.nonce += 1;
        CryptoHash::try_from(env::sha256(&data[..])).unwrap()
    }

    /// Record order id to make sure it is not re-used later for another deposit.
    fn record_order_id(&mut self, order_id: &CryptoHash) -> Balance {
        let initial_storage = env::storage_usage();
        self.used_events.insert(order_id);
        let current_storage = env::storage_usage();
        let required_deposit =
            Balance::from(current_storage - initial_storage) * env::storage_byte_cost();

        env::log_str(format!("RecordOrderId:{}", hex::encode(order_id)).as_str());
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

    fn is_native_token(&self, token: &AccountId) -> bool {
        token.eq(&self.wrapped_token)
    }

    fn native_token_address(&self) -> (Vec<u8>, String) {
        (
            Vec::from(self.wrapped_token.to_string()),
            self.wrapped_token.to_string(),
        )
    }

    fn check_to_account(&mut self, to: Vec<u8>, chain_id: u128) {
        match self.get_chain_type(chain_id.into()) {
            EvmChain => {
                assert_eq!(
                    20,
                    to.len(),
                    "address length is incorrect for evm chain type"
                )
            }
            _ => panic_str(format!("unknown chain type for chain {}", chain_id).as_str()),
        }
    }
}

admin_controlled::impl_admin_controlled!(MAPOServiceV2, paused);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use map_light_client::header::Header;
    use map_light_client::proof::Receipt;
    use map_light_client::G2;
    use near_sdk::json_types::U64;
    use near_sdk::{env::sha256, test_utils::VMContextBuilder, testing_env};
    use std::convert::TryInto;
    use uint::rustc_hex::ToHex;

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
        (
            "alice.near".parse().unwrap(),
            hex::decode("ab175474e89094c44da98b954eedeac495271d0f").unwrap(),
        )
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

    fn wrap_token() -> AccountId {
        "wrap.near".parse().unwrap()
    }

    fn mcs_token() -> (AccountId, String) {
        (
            "mcs.near".parse().unwrap(),
            "cb175474e89094c44da98b954eedeac495271d0f".to_string(),
        )
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
                receipt_type: U128(0),
                post_state_or_status: vec![],
                cumulative_gas_used: U64(0),
                bloom: [0; 256],
                logs: vec![],
            },
            key_index: vec![],
            proof: vec![],
        }
    }

    fn mcs_contract() -> MAPOServiceV2 {
        MAPOServiceV2 {
            map_client_account: prover().parse().unwrap(),
            map_bridge_address: validate_eth_address(map_bridge_address()),
            mcs_tokens: UnorderedMap::new(b"t".to_vec()),
            fungible_tokens: UnorderedMap::new(b"f".to_vec()),
            fungible_tokens_storage_balance: UnorderedMap::new(b"s".to_vec()),
            token_decimals: UnorderedMap::new(b"d".to_vec()),
            native_to_chains: Default::default(),
            chain_id_type_map: UnorderedMap::new(b"c".to_vec()),
            used_events: UnorderedSet::new(b"u".to_vec()),
            owner: env::signer_account_id(),
            mcs_storage_balance_min: STORAGE_BALANCE,
            wrapped_token: wrap_token(),
            near_chain_id: NEAR_CHAIN_ID, // 1313161555 for testnet
            map_chain_id: 0,
            nonce: 0,
            paused: Mask::default(),
            registered_tokens: UnorderedMap::new(b"r".to_vec()),
            ref_exchange: "ref.testnet".parse().unwrap(),
            core_idle: vec![],
            core_total: vec![],
            amount_out: Default::default(),
            lost_found: UnorderedMap::new(b"l".to_vec()),
        }
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
    #[should_panic]
    fn test_transfer_out_token_no_to_chain() {
        let token = mcs_token().0;
        let from = alice().0;
        let to = alice().1;
        let mut contract = mcs_contract();

        set_env!(
            predecessor_account_id: alice().0
        );

        contract.registered_tokens.insert(&token, &true);

        set_env!(
            current_account_id: map_cross_chain_service(),
            predecessor_account_id: format!("{}.{}", token, map_cross_chain_service())
        );

        contract.transfer_out_token(token, to, U128(1_000), ETH_CHAIN_ID.into());
    }
}
