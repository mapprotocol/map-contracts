use crate::*;
use near_sdk::Promise;
use std::clone;

/// Gas to initialize MCSToken contract.
const MCS_TOKEN_NEW: Gas = Gas(10_000_000_000_000);
/// Gas to call storage_deposit_for_contracts on mcs contract
const STORAGE_DEPOSIT_FOR_MCS_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call ft_metadata on ext fungible contract
const FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

const GET_FT_METADATA_GAS: Gas = Gas(20_000_000_000_000);

/// Gas to call finish_add_fungible_token_to_chain method.
const FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS: Gas = Gas(20_000_000_000_000);

/// Gas to call storage_balance_bounds on ext fungible contract
const STORAGE_BALANCE_BOUNDS_GAS: Gas = Gas(30_000_000_000_000);
/// Amount of gas used by set_metadata in the mcs, without taking into account
/// the gas consumed by the promise.
const OUTER_SET_METADATA_GAS: Gas = Gas(15_000_000_000_000);

/// Gas to call callback_add_native_to_chain on current contract
const CALLBACK_ADD_NATIVE_TO_CHAIN: Gas = Gas(5_000_000_000_000);

#[near_bindgen]
impl MapCrossChainService {
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
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.mcs_tokens.get(&address).is_some(),
            "token {} is not mcs token",
            address
        );

        if let Some(value) = decimals {
            self.token_decimals.insert(&address, &value);
        }

        ext_mcs_token::ext(address.parse().unwrap())
            .with_static_gas(env::prepaid_gas() - OUTER_SET_METADATA_GAS)
            .set_metadata(name, symbol, reference, reference_hash, decimals, icon)
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
                >= MCS_TOKEN_INIT_BALANCE
                    + self.mcs_storage_transfer_in_required * (self.core_total.len() as u128 + 2)
                    + env::storage_byte_cost() * (current_storage - initial_storage),
            "Not enough attached deposit to complete mcs token creation"
        );

        let token: AccountId = address.parse().unwrap();
        let mut promise = Promise::new(token.clone())
            .create_account()
            .transfer(MCS_TOKEN_INIT_BALANCE)
            .deploy_contract(MCS_TOKEN_BINARY.to_vec())
            .function_call(
                "new".to_string(),
                json!({ "owner": self.owner})
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                NO_DEPOSIT,
                MCS_TOKEN_NEW,
            );

        // add storage deposit for mcs/core/ref exchange account
        for core in self.core_total.clone() {
            promise = promise.then(
                ext_fungible_token::ext(token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(self.mcs_storage_transfer_in_required)
                    .storage_deposit(Some(core), Some(true)),
            );
        }
        promise
            .then(
                ext_fungible_token::ext(token.clone())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(self.mcs_storage_transfer_in_required)
                    .storage_deposit(Some(env::current_account_id()), Some(true)),
            )
            .then(
                ext_fungible_token::ext(token)
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(self.mcs_storage_transfer_in_required)
                    .storage_deposit(Some(self.ref_exchange.clone()), Some(true)),
            )
    }

    pub fn add_mcs_token_to_chain(&mut self, token: String, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        let mut to_chain_set = self
            .mcs_tokens
            .get(&token)
            .expect(format!("token {} is not supported", token).as_str());
        to_chain_set.insert(to_chain.into());
        self.mcs_tokens.insert(&token, &to_chain_set);
    }

    pub fn remove_mcs_token_to_chain(&mut self, token: String, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        let mut to_chain_set = self
            .mcs_tokens
            .get(&token)
            .expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain.into());
        self.mcs_tokens.insert(&token, &to_chain_set);
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

    pub fn valid_mcs_token_out(&self, token: &String, to_chain: U128) -> bool {
        let to_chain_set_wrap = self.mcs_tokens.get(token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain.into())
    }

    pub fn add_fungible_token_to_chain(
        &mut self,
        token: String,
        to_chain: U128,
    ) -> PromiseOrValue<()> {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.mcs_tokens.get(&token).is_none(),
            "token name {} exists in mcs token",
            token
        );

        if self.fungible_tokens_storage_balance.get(&token).is_none() {
            ext_fungible_token::ext(token.parse().unwrap())
                .with_static_gas(STORAGE_BALANCE_BOUNDS_GAS)
                .storage_balance_bounds()
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(
                            GET_FT_METADATA_GAS
                                + FT_METADATA_GAS
                                + STORAGE_DEPOSIT_FOR_MCS_GAS
                                + STORAGE_DEPOSIT_GAS
                                + FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS,
                        )
                        .get_fungible_token_metadata(token, to_chain),
                )
                .into()
        } else {
            let mut to_chain_set = self.fungible_tokens.get(&token).unwrap_or_default();
            to_chain_set.insert(to_chain.into());
            self.fungible_tokens.insert(&token, &to_chain_set);
            PromiseOrValue::Value(())
        }
    }

    pub fn remove_fungible_token_to_chain(&mut self, token: String, to_chain: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        let mut to_chain_set = self
            .fungible_tokens
            .get(&token)
            .expect(format!("token {} is not supported", token).as_str());
        to_chain_set.remove(&to_chain.into());
        if to_chain_set.is_empty() {
            self.fungible_tokens.remove(&token);
        } else {
            self.fungible_tokens.insert(&token, &to_chain_set);
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

    pub fn valid_fungible_token_out(&self, token: &String, to_chain: U128) -> bool {
        let to_chain_set_wrap = self.fungible_tokens.get(token);
        if to_chain_set_wrap.is_none() {
            return false;
        }
        let to_chain_set = to_chain_set_wrap.unwrap();

        to_chain_set.contains(&to_chain.into())
    }

    pub fn add_native_to_chain(&mut self, to_chain: U128) -> bool {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        self.native_to_chains.insert(to_chain.into())
    }

    // #[private]
    // pub fn callback_add_native_to_chain(&mut self, to_chain: U128) {
    //     assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
    //
    //     match env::promise_result(0) {
    //         PromiseResult::Successful(x) => self.native_to_chains.insert(to_chain.into()),
    //         _ => panic_str(&*format!("add wrap token to chain {} failed", to_chain.0)),
    //     };
    // }

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

    #[private]
    pub fn get_fungible_token_metadata(&self, token: String, to_chain: U128) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        let bounds = match env::promise_result(0) {
            PromiseResult::Successful(x) => {
                serde_json::from_slice::<StorageBalanceBounds>(&x).unwrap()
            }
            _ => panic_str(&*format!(
                "get storage_balance_bounds of token {} failed",
                token
            )),
        };

        ext_fungible_token::ext(token.parse().unwrap())
            .with_static_gas(FT_METADATA_GAS)
            .ft_metadata()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(
                        STORAGE_DEPOSIT_FOR_MCS_GAS
                            + STORAGE_DEPOSIT_GAS
                            + FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS,
                    )
                    .storage_deposit_for_contracts(token, to_chain, bounds.min),
            )
    }

    #[private]
    pub fn storage_deposit_for_contracts(
        &self,
        token: String,
        to_chain: U128,
        min_bound: U128,
    ) -> Promise {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        let metadata = match env::promise_result(0) {
            PromiseResult::Successful(x) => {
                serde_json::from_slice::<FungibleTokenMetadata>(&x).unwrap()
            }
            _ => panic_str(&*format!("get metadata of token {} failed", token)),
        };

        for core in self.core_total.clone() {
            ext_fungible_token::ext(token.parse().unwrap())
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .with_attached_deposit(min_bound.into())
                .storage_deposit(Some(core), Some(true));
        }
        ext_fungible_token::ext(token.parse().unwrap())
            .with_static_gas(STORAGE_DEPOSIT_GAS)
            .with_attached_deposit(min_bound.into())
            .storage_deposit(Some(env::current_account_id()), Some(true))
            .then(
                ext_fungible_token::ext(token.parse().unwrap())
                    .with_static_gas(STORAGE_DEPOSIT_GAS)
                    .with_attached_deposit(min_bound.into())
                    .storage_deposit(Some(self.ref_exchange.clone()), Some(true)),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FINISH_ADD_FUNGIBLE_TOKEN_TO_CHAINGAS)
                    .finish_add_fungible_token_to_chain(
                        token,
                        to_chain,
                        min_bound,
                        metadata.decimals,
                    ),
            )
    }

    #[private]
    pub fn finish_add_fungible_token_to_chain(
        &mut self,
        token: String,
        to_chain: U128,
        min_bound: U128,
        decimals: u8,
    ) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::Successful(x) => {}
            _ => panic_str(&*format!(
                "storage deposit to token {} for mcs/core/ref-exchanger failed",
                token
            )),
        };

        let mut to_chain_set = self.fungible_tokens.get(&token).unwrap_or_default();
        to_chain_set.insert(to_chain.into());
        self.fungible_tokens.insert(&token, &to_chain_set);
        self.fungible_tokens_storage_balance
            .insert(&token, &min_bound.into());
        self.token_decimals.insert(&token, &decimals);
    }
}
