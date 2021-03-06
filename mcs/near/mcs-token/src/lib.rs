use admin_controlled::Mask;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault,
    Promise, PromiseOrValue, StorageUsage,
};
use std::convert::TryInto;

near_sdk::setup_alloc!();

const NO_DEPOSIT: Balance = 0;

/// Gas to call finish withdraw method on factory.
const FINISH_WITHDRAW_GAS: Gas = Gas(50_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MCSToken {
    controller: AccountId,
    token: FungibleToken,
    name: String,
    symbol: String,
    reference: String,
    reference_hash: Base64VecU8,
    decimals: u8,
    paused: Mask,
    #[cfg(feature = "migrate_icon")]
    icon: Option<String>,
}

const PAUSE_WITHDRAW: Mask = 1 << 0;

#[ext_contract(ext_map_cross_chain_service)]
pub trait ExtMapCrossChainService {
    #[result_serializer(borsh)]
    fn finish_withdraw(
        &self,
        #[serializer(borsh)] amount: Balance,
        #[serializer(borsh)] recipient: AccountId,
    ) -> Promise;
}

#[near_bindgen]
impl MCSToken {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            controller: env::predecessor_account_id(),
            token: FungibleToken::new(b"t".to_vec()),
            name: String::default(),
            symbol: String::default(),
            reference: String::default(),
            reference_hash: Base64VecU8(vec![]),
            decimals: 0,
            paused: Mask::default(),
            #[cfg(feature = "migrate_icon")]
            icon: None,
        }
    }

    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.controller,
            "Only controller can call mint"
        );

        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
    }

    #[payable]
    pub fn withdraw(&mut self, amount: U128, recipient: String) -> Promise {
        self.check_not_paused(PAUSE_WITHDRAW);

        assert_one_yocto();
        Promise::new(env::predecessor_account_id()).transfer(1);

        self.token
            .internal_withdraw(&env::predecessor_account_id(), amount.0);

        // ext_map_cross_chain_service::finish_withdraw(
        //     amount.into(),
        //     recipient,
        //     &self.controller,
        //     NO_DEPOSIT,
        //     FINISH_WITHDRAW_GAS,
        // )

        ext_map_cross_chain_service::ext(self.controller.clone())
            .with_static_gas(FINISH_WITHDRAW_GAS)
            .finish_withdraw(amount.0, recipient.parse().unwrap())
    }

    #[payable]
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.controller,
            "Only controller can call burn"
        );

        self.token
            .internal_withdraw(&account_id, amount.into());
    }

    pub fn account_storage_usage(&self) -> StorageUsage {
        self.token.account_storage_usage
    }

    /// Return true if the caller is either controller or self
    pub fn controller_or_self(&self) -> bool {
        let caller = env::predecessor_account_id();
        caller == self.controller || caller == env::current_account_id()
    }
}

near_contract_standards::impl_fungible_token_core!(MCSToken, token);
near_contract_standards::impl_fungible_token_storage!(MCSToken, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for MCSToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            #[cfg(feature = "migrate_icon")]
            icon: self.icon.clone(),
            #[cfg(not(feature = "migrate_icon"))]
            icon: None,
            reference: Some(self.reference.clone()),
            reference_hash: Some(self.reference_hash.clone()),
            decimals: self.decimals,
        }
    }
}

admin_controlled::impl_admin_controlled!(MCSToken, paused);