use crate::token_receiver::SwapInfo;
use crate::*;
use admin_controlled::AdminControlled;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Gas, PromiseResult};
use std::collections::HashMap;

const FT_TRANSFER_CALL_CORE_GAS: Gas = Gas(210_000_000_000_000);
const CALL_CORE_SWAP_IN_DIRECTLY_GAS: Gas = Gas(180_000_000_000_000);
const CALL_CORE_SWAP_OUT_DIRECTLY_GAS: Gas = Gas(175_000_000_000_000);
/// Gas to call callback_swap_out_token method.
const CALLBACK_SWAP_OUT_TOKEN_GAS: Gas =
    Gas(15_000_000_000_000 + BURN_GAS.0 + FINISH_TOKEN_OUT_GAS.0);

const CALLBACK_DO_SWAP_GAS: Gas =
    Gas(20_000_000_000_000 + CALL_CORE_SWAP_IN_DIRECTLY_GAS.0 + CALLBACK_POST_SWAP_IN_GAS.0);

const CALLBACK_ADD_BUTTER_CORE_GAS: Gas = Gas(5_000_000_000_000);

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CoreSwapMessage {
    /// List of sequential actions.
    pub actions: Vec<Action>,
    pub target_account: AccountId,
    pub target_token: Option<AccountId>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    Swap(SwapAction),
}

/// Single swap action.
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

#[near_bindgen]
impl MAPOServiceV2 {
    pub fn add_butter_core(&mut self, butter_core: AccountId) -> Promise {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        if self.core_total.contains(&butter_core) {
            panic_str("core already exists!")
        }

        let mut promise = Promise::new(env::current_account_id());
        for token in self.fungible_tokens.keys() {
            promise = promise.then(
                ext_fungible_token::ext(token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(
                        self.fungible_tokens_storage_balance.get(&token).unwrap(),
                    )
                    .storage_deposit(Some(butter_core.clone()), Some(true)),
            );
        }

        for token in self.mcs_tokens.keys() {
            promise = promise.then(
                ext_fungible_token::ext(token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(self.mcs_storage_balance_min)
                    .storage_deposit(Some(butter_core.clone()), Some(true)),
            );
        }
        promise
            .then(
                ext_fungible_token::ext(self.wrapped_token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(self.mcs_storage_balance_min)
                    .storage_deposit(Some(butter_core.clone()), Some(true)),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(CALLBACK_ADD_BUTTER_CORE_GAS)
                    .callback_add_butter_core(butter_core),
            )
    }

    #[private]
    pub fn callback_add_butter_core(&mut self, butter_core: AccountId) {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => {
                self.core_idle.push(butter_core.clone());
                self.core_total.push(butter_core)
            }
            PromiseResult::Failed => {
                panic_str("register core to token failed");
            }
        }
    }

    pub fn get_butter_core(&self) -> HashMap<AccountId, String> {
        let mut map: HashMap<AccountId, String> = HashMap::new();

        for core in self.core_idle.clone() {
            map.insert(core, "idle".to_string());
        }
        for core in self.core_total.clone() {
            map.entry(core).or_insert_with(|| "working".to_string());
        }

        map
    }

    pub fn reset_butter_core(&mut self, core: AccountId) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        self.core_idle.push(core)
    }

    pub fn clean_idle_core(&mut self) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        for core in self.core_idle.clone() {
            self.core_idle.retain(|x| *x != core);
            self.core_total.retain(|x| *x != core);
        }
    }

    pub fn get_ref_exchange(&self) -> AccountId {
        self.ref_exchange.clone()
    }

    #[private]
    pub fn process_token_swap_out(
        &mut self,
        to_chain: U128,
        src_token: String,
        token: AccountId,
        from: AccountId,
        to: Vec<u8>,
        amount: U128,
        swap_info: SwapInfo,
    ) -> PromiseOrValue<U128> {
        self.check_not_paused(PAUSE_SWAP_OUT_TOKEN);
        self.check_swap_param(&token, amount, &swap_info);

        if swap_info.src_swap.is_empty() {
            self.swap_out_token(
                to_chain,
                src_token,
                amount,
                token,
                from,
                to,
                amount,
                swap_info.dst_swap,
            )
        } else {
            let core = self
                .core_idle
                .pop()
                .unwrap_or_else(|| panic_str("no idle core!"));

            let last_action = swap_info.src_swap.last().unwrap().to_swap_action();
            self.check_token_to_chain(&last_action.token_out, to_chain);
            self.check_to_account(to.clone(), to_chain.into());
            self.check_amount(&last_action.token_out, last_action.min_amount_out.into());

            let actions = swap_info
                .clone()
                .src_swap
                .into_iter()
                .map(|param| Action::Swap(param.to_swap_action()))
                .collect();
            let core_swap_msg = CoreSwapMessage {
                actions,
                target_account: env::current_account_id(),
                target_token: None,
            };
            // let msg = serde_json::to_string(&core_swap_msg).unwrap();
            // call core to do swap
            // ext_ft_core::ext(token)
            //     .with_static_gas(FT_TRANSFER_CALL_CORE_GAS)
            //     .with_attached_deposit(1)
            //     .ft_transfer_call(core.clone(), amount, None, msg)
            //     .then(
            //         Self::ext(env::current_account_id())
            //             .with_static_gas(CALLBACK_SWAP_OUT_TOKEN_GAS)
            //             .callback_swap_out_token(
            //                 core,
            //                 to_chain,
            //                 src_token,
            //                 last_action.token_out,
            //                 from,
            //                 to,
            //                 amount,
            //                 swap_info,
            //             ),
            //     )
            //     .into()
            self.call_core_swap_out_directly(
                to_chain,
                src_token,
                last_action.token_out,
                token,
                from,
                to,
                amount,
                core,
                core_swap_msg,
                swap_info,
            )
        }
    }

    #[private]
    pub fn callback_swap_out_token(
        &mut self,
        core: AccountId,
        to_chain: U128,
        src_token: String,
        token_out: AccountId,
        from: AccountId,
        to: Vec<u8>,
        amount: U128,
        swap_info: SwapInfo,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        self.core_idle.push(core.clone());
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(x) => {
                let (used_amount, _) = serde_json::from_slice::<(U128, U128)>(&x).unwrap();
                if amount != used_amount {
                    log!("used amount is unexpected, swap in core failed, expected: {:?}, actual: {:?}!", amount, used_amount);
                    PromiseOrValue::Value(U128(amount.0 - used_amount.0))
                } else {
                    // TODO: revert state
                    let amount_out = self.amount_out.remove(&core).unwrap_or_else(|| {
                        panic_str("unexpected! mos didn't receive amount out from core")
                    });
                    self.swap_out_token(
                        to_chain,
                        src_token,
                        amount,
                        token_out,
                        from,
                        to,
                        amount_out,
                        swap_info.dst_swap,
                    )
                }
            }
            PromiseResult::Failed => {
                // ft_transfer_call fails only if the prepaid gas is not enough, so the failure will not be triggered because we always give it enough static gas
                panic_str("call core to do swap failed");
            }
        }
    }

    #[payable]
    #[private]
    pub fn callback_do_swap(
        &mut self,
        core: AccountId,
        token_in: AccountId,
        target_account: AccountId,
        amount: U128,
        msg: CoreSwapMessage,
        order_id: CryptoHash,
        token_in_storage_balance: Balance,
    ) -> Promise {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        let ret_deposit = env::attached_deposit();
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(x) => {
                ext_butter_core::ext(core.clone())
                    .with_static_gas(CALL_CORE_SWAP_IN_DIRECTLY_GAS)
                    .swap(amount, msg.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(CALLBACK_POST_SWAP_IN_GAS)
                            .with_attached_deposit(ret_deposit)
                            .callback_post_swap_in(
                                core,
                                token_in,
                                msg.target_token.unwrap(),
                                target_account,
                                amount,
                                // ret_deposit,
                                order_id,
                                token_in_storage_balance,
                            ),
                    )
            }
            PromiseResult::Failed => {
                let err_msg = format!(
                    "[FAILURE] mint or transfer token to core failed  {:?}",
                    token_in
                );
                self.core_idle.push(core);
                self.revert_state(ret_deposit, Some(order_id), err_msg)
            }
        }
    }

    #[payable]
    #[private]
    pub fn callback_post_swap_in(
        &mut self,
        core: AccountId,
        token_in: AccountId,
        token_out: AccountId,
        to: AccountId,
        amount: U128,
        // ret_deposit: Balance,
        order_id: CryptoHash,
        token_in_storage_balance: Balance,
    ) -> Promise {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        let ret_deposit = env::attached_deposit();
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(x) => {
                let (used_amount, amount_out) = serde_json::from_slice::<(U128, U128)>(&x).unwrap();
                if amount != used_amount {
                    log!("[SWAP FAILURE] call core to do swap failed, used amount is unexpected, expected: {:?}, actual: {:?}, transfer token in to user!", amount, used_amount);
                    if used_amount.0 == 0 {
                        ext_fungible_token::ext(token_in.clone())
                            .with_static_gas(STORAGE_DEPOSIT_GAS)
                            .with_attached_deposit(token_in_storage_balance)
                            .storage_deposit(Some(to.clone()), Some(true))
                            .then(
                                ext_ft_core::ext(token_in.clone())
                                    .with_static_gas(FT_TRANSFER_GAS)
                                    .with_attached_deposit(1)
                                    .ft_transfer(to, amount, None),
                            );

                        SwapInEvent {
                            order_id,
                            token_out: token_in,
                            amount_out: amount,
                        }
                        .emit();
                        self.core_idle.push(core);
                        log!(
                            "prepaid gas: {:?}, used gas {:?}",
                            env::prepaid_gas(),
                            env::used_gas()
                        );
                        Promise::new(env::signer_account_id())
                            .transfer(ret_deposit - token_in_storage_balance)
                    } else {
                        // amount < used amount
                        self.revert_state(
                            ret_deposit,
                            None,
                            "used amount != amount in && used amount != 0, please check core state!".to_string(),
                        )
                    }
                } else {
                    SwapInEvent {
                        order_id,
                        token_out,
                        amount_out,
                    }
                    .emit();
                    self.core_idle.push(core);
                    Promise::new(env::signer_account_id()).transfer(ret_deposit)
                }
            }
            PromiseResult::Failed => {
                let err_msg = format!("[SWAP FAILURE] call core to do swap in failed, maybe mos doesn't have enough token {:?}", token_in);
                self.revert_state(ret_deposit, Some(order_id), err_msg)
            }
        }
    }
}

impl MAPOServiceV2 {
    fn check_swap_param(&self, token: &AccountId, amount: U128, swap_info: &SwapInfo) {
        if !swap_info.src_swap.is_empty() {
            let actions: Vec<SwapAction> = swap_info
                .src_swap
                .clone()
                .into_iter()
                .map(|param| param.to_swap_action())
                .collect();

            let mut prev_token = token.clone();
            for action in actions {
                assert!(action.amount_in.is_none(), "amount in should be 0");
                assert_eq!(
                    prev_token, action.token_in,
                    "token in should be equal to prev token out"
                );
                prev_token = action.token_out;
            }
        }
        self.check_amount(token, amount.0);
    }

    // pub fn call_core_swap_in(
    //     &self,
    //     core: AccountId,
    //     token_in: AccountId,
    //     target_account: AccountId,
    //     amount: U128,
    //     msg: String,
    //     ret_deposit: Balance,
    //     order_id: CryptoHash,
    //     token_in_storage_balance: Balance,
    // ) -> Promise {
    //     ext_ft_core::ext(token_in.clone())
    //         .with_static_gas(FT_TRANSFER_CALL_CORE_GAS)
    //         .with_attached_deposit(1)
    //         .ft_transfer_call(core.clone(), amount, None, msg)
    //         .then(
    //             Self::ext(env::current_account_id())
    //                 .with_attached_deposit(ret_deposit)
    //                 .with_static_gas(CALLBACK_POST_SWAP_IN_GAS)
    //                 .callback_post_swap_in(
    //                     core,
    //                     token_in,
    //                     target_account,
    //                     amount,
    //                     // ret_deposit,
    //                     order_id,
    //                     token_in_storage_balance,
    //                 ),
    //         )
    // }

    pub fn call_core_swap_in_directly(
        &self,
        core: AccountId,
        token_in: AccountId,
        target_account: AccountId,
        amount: U128,
        msg: CoreSwapMessage,
        ret_deposit: Balance,
        order_id: CryptoHash,
        token_in_storage_balance: Balance,
    ) -> Promise {
        if self.mcs_tokens.get(&token_in).is_some() {
            ext_mcs_token::ext(token_in.clone())
                .with_static_gas(MINT_GAS)
                .mint(core.clone(), amount)
        } else {
            ext_ft_core::ext(token_in.clone())
                .with_static_gas(FT_TRANSFER_GAS)
                .with_attached_deposit(1)
                .ft_transfer(core.clone(), amount, None)
        }
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(CALLBACK_DO_SWAP_GAS)
                .with_attached_deposit(ret_deposit)
                .callback_do_swap(
                    core,
                    token_in,
                    target_account,
                    amount,
                    msg,
                    // ret_deposit,
                    order_id,
                    token_in_storage_balance,
                ),
        )
    }

    fn call_core_swap_out_directly(
        &self,
        to_chain: U128,
        src_token: String,
        token_out: AccountId,
        token: AccountId,
        from: AccountId,
        to: Vec<u8>,
        amount: U128,
        core: AccountId,
        msg: CoreSwapMessage,
        swap_info: SwapInfo,
    ) -> PromiseOrValue<U128> {
        ext_ft_core::ext(token)
            .with_static_gas(FT_TRANSFER_GAS)
            .with_attached_deposit(1)
            .ft_transfer(core.clone(), amount, None)
            .then(
                ext_butter_core::ext(core.clone())
                    .with_static_gas(CALL_CORE_SWAP_OUT_DIRECTLY_GAS)
                    .swap(amount, msg),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(CALLBACK_SWAP_OUT_TOKEN_GAS)
                    .callback_swap_out_token(
                        core, to_chain, src_token, token_out, from, to, amount, swap_info,
                    ),
            )
            .into()
    }
}
