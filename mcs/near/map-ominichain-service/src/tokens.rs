use crate::*;
use near_sdk::Promise;

/// Gas to call callback_storage_deposit_for_contracts on mcs contract
const CALLBACK_STORAGE_DEPOSIT_FOR_CONTRACTS_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call ft_metadata on ext fungible contract
const FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

const CALLBACK_GET_FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call callback_finish_register_token method.
const CALLBACK_FINISH_REGISTER_TOKEN_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call storage_balance_bounds on ext fungible contract
const STORAGE_BALANCE_BOUNDS_GAS: Gas = Gas(30_000_000_000_000);

#[near_bindgen]
impl MAPOServiceV2 {
    #[payable]
    pub fn register_token(&mut self, token: AccountId, mintable: bool) -> Promise {
        assert!(
            self.registered_tokens.get(&token).is_none(),
            "token {} has already been registered!",
            token
        );
        let count = (self.core_total.len() + 2) as u128;
        let deposit = env::attached_deposit();

        if mintable {
            let required = count * self.mcs_storage_balance_min;
            assert!(
                deposit >= required,
                "not enough attached deposit, required: {}, actual: {}",
                required,
                deposit
            );

            ext_fungible_token::ext(token.clone())
                .with_static_gas(FT_METADATA_GAS)
                .ft_metadata()
                .then(
                    Self::ext(env::current_account_id())
                        .with_attached_deposit(deposit)
                        .with_static_gas(
                            CALLBACK_STORAGE_DEPOSIT_FOR_CONTRACTS_GAS
                                + STORAGE_DEPOSIT_GAS * count as u64
                                + CALLBACK_FINISH_REGISTER_TOKEN_GAS,
                        )
                        .callback_storage_deposit_for_contracts(
                            token,
                            self.mcs_storage_balance_min.into(),
                            true,
                        ),
                )
        } else {
            ext_fungible_token::ext(token.clone())
                .with_static_gas(STORAGE_BALANCE_BOUNDS_GAS)
                .storage_balance_bounds()
                .then(
                    Self::ext(env::current_account_id())
                        .with_attached_deposit(deposit)
                        .with_static_gas(
                            CALLBACK_GET_FT_METADATA_GAS
                                + FT_METADATA_GAS
                                + CALLBACK_STORAGE_DEPOSIT_FOR_CONTRACTS_GAS
                                + STORAGE_DEPOSIT_GAS * count as u64
                                + CALLBACK_FINISH_REGISTER_TOKEN_GAS,
                        )
                        .callback_get_ft_metadata(token),
                )
        }
    }

    #[private]
    #[payable]
    pub fn callback_storage_deposit_for_contracts(
        &mut self,
        token: AccountId,
        min_bound: U128,
        mintable: bool,
    ) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::Successful(x) => {
                let metadata = serde_json::from_slice::<FungibleTokenMetadata>(&x).unwrap();
                let deposit =
                    env::attached_deposit() - (self.core_total.len() + 2) as u128 * min_bound.0;

                let mut promise = ext_fungible_token::ext(token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(min_bound.into())
                    .storage_deposit(Some(env::current_account_id()), Some(true))
                    .and(
                        ext_fungible_token::ext(token.clone())
                            .with_static_gas(STORAGE_DEPOSIT_GAS)
                            .with_attached_deposit(min_bound.into())
                            .storage_deposit(Some(self.ref_exchange.clone()), Some(true)),
                    );
                for core in self.core_total.clone() {
                    promise = promise.and(
                        ext_fungible_token::ext(token.clone())
                            .with_static_gas(STORAGE_DEPOSIT_GAS)
                            .with_attached_deposit(min_bound.into())
                            .storage_deposit(Some(core), Some(true)),
                    );
                }

                promise.then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(CALLBACK_FINISH_REGISTER_TOKEN_GAS)
                        .with_attached_deposit(deposit)
                        .callback_finish_register_token(
                            token,
                            min_bound,
                            metadata.decimals,
                            mintable,
                        ),
                )
            }
            _ => self.revert_state(
                env::attached_deposit(),
                None,
                format!("get metadata of token {} failed", token),
            ),
        }
    }

    #[private]
    #[payable]
    pub fn callback_get_ft_metadata(&mut self, token: AccountId) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        let deposit = env::attached_deposit();

        match env::promise_result(0) {
            PromiseResult::Successful(x) => {
                let bounds = serde_json::from_slice::<StorageBalanceBounds>(&x).unwrap();
                let count = (self.core_total.len() + 2) as u128;
                let required = count * bounds.min.0;
                if deposit < required {
                    let msg = format!(
                        "not enough attached deposit, required: {}, actual: {}",
                        required, deposit
                    );
                    return self.revert_state(deposit, None, msg);
                }

                ext_fungible_token::ext(token.clone())
                    .with_static_gas(FT_METADATA_GAS)
                    .ft_metadata()
                    .then(
                        Self::ext(env::current_account_id())
                            .with_attached_deposit(deposit)
                            .with_static_gas(
                                CALLBACK_STORAGE_DEPOSIT_FOR_CONTRACTS_GAS
                                    + STORAGE_DEPOSIT_GAS * count as u64
                                    + CALLBACK_FINISH_REGISTER_TOKEN_GAS,
                            )
                            .callback_storage_deposit_for_contracts(token, bounds.min, false),
                    )
            }
            _ => self.revert_state(
                deposit,
                None,
                format!("get ft metadata of token {} failed", token),
            ),
        }
    }

    #[private]
    #[payable]
    pub fn callback_finish_register_token(
        &mut self,
        token: AccountId,
        min_bound: U128,
        decimals: u8,
        mintable: bool,
    ) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                self.registered_tokens.insert(&token, &mintable);
                self.token_decimals.insert(&token.to_string(), &decimals);
                if !mintable {
                    self.fungible_tokens_storage_balance
                        .insert(&token.to_string(), &min_bound.into());
                }

                Promise::new(env::signer_account_id()).transfer(env::attached_deposit())
            }
            _ => self.revert_state(
                env::attached_deposit(),
                None,
                format!(
                    "storage deposit to token {} for mcs/core/ref-exchange failed",
                    token
                ),
            ),
        }
    }

    pub fn add_mcs_token_to_chain(&mut self, token: AccountId, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        if let Some(mintable) = self.registered_tokens.get(&token) {
            assert!(mintable, "token {} is not mintable", token);

            let mut to_chain_set = self.mcs_tokens.get(&token.to_string()).unwrap_or_default();
            to_chain_set.insert(to_chain.into());
            self.mcs_tokens.insert(&token.to_string(), &to_chain_set);
        } else {
            panic_str(format!("token {} has not been registered!", token).as_str())
        }
    }

    pub fn remove_mcs_token_to_chain(&mut self, token: AccountId, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        if let Some(mut to_chain_set) = self.mcs_tokens.remove(&token.to_string()) {
            to_chain_set.remove(&to_chain.into());
            if !to_chain_set.is_empty() {
                self.mcs_tokens.insert(&token.to_string(), &to_chain_set);
            }
        } else {
            panic_str(format!("token {} has not no target chains", token).as_str())
        }
    }

    /// Return all registered mcs tokens
    pub fn get_mcs_tokens(&self) -> Vec<(String, Vec<U128>)> {
        let mut ret: Vec<(String, Vec<U128>)> = Vec::new();

        for (x, y) in self.mcs_tokens.to_vec() {
            let y = y.into_iter().map(U128).collect();
            ret.push((x, y))
        }

        ret
    }

    pub fn valid_mcs_token_out(&self, token: &AccountId, to_chain: U128) -> bool {
        if let Some(to_chain_set) = self.mcs_tokens.get(&token.to_string()) {
            to_chain_set.contains(&to_chain.into())
        } else {
            false
        }
    }

    pub fn add_fungible_token_to_chain(&mut self, token: AccountId, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        if let Some(mintable) = self.registered_tokens.get(&token) {
            assert!(!mintable, "token {} is mintable", token);

            let mut to_chain_set = self
                .fungible_tokens
                .get(&token.to_string())
                .unwrap_or_default();
            to_chain_set.insert(to_chain.into());
            self.fungible_tokens
                .insert(&token.to_string(), &to_chain_set);
        } else {
            panic_str(format!("token {} has not been registered!", token).as_str())
        }
    }

    pub fn remove_fungible_token_to_chain(&mut self, token: AccountId, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        if let Some(mut to_chain_set) = self.fungible_tokens.remove(&token.to_string()) {
            to_chain_set.remove(&to_chain.into());
            if !to_chain_set.is_empty() {
                self.fungible_tokens
                    .insert(&token.to_string(), &to_chain_set);
            }
        } else {
            panic_str(format!("token {} has not no target chains", token).as_str())
        }
    }

    /// Return all registered fungible tokens (not mcs token)
    pub fn get_fungible_tokens(&self) -> Vec<(String, Vec<U128>)> {
        let mut ret: Vec<(String, Vec<U128>)> = Vec::new();

        for (x, y) in self.fungible_tokens.to_vec() {
            let y = y.into_iter().map(U128).collect();
            ret.push((x, y))
        }

        ret
    }

    pub fn valid_fungible_token_out(&self, token: &AccountId, to_chain: U128) -> bool {
        if let Some(to_chain_set) = self.fungible_tokens.get(&token.to_string()) {
            to_chain_set.contains(&to_chain.into())
        } else {
            false
        }
    }

    pub fn add_native_to_chain(&mut self, to_chain: U128) -> bool {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        self.native_to_chains.insert(to_chain.into())
    }

    pub fn remove_native_to_chain(&mut self, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        self.native_to_chains.remove(&to_chain.into());
    }
    /// Return all native token to chains
    pub fn get_native_token_to_chains(&self) -> Vec<U128> {
        self.native_to_chains
            .clone()
            .into_iter()
            .map(U128)
            .collect()
    }
}
