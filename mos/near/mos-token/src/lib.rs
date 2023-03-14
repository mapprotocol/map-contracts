use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{env, near_bindgen, AccountId, Gas, PanicOnDefault, PromiseOrValue, StorageUsage};

const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(15_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MCSToken {
    /// Controller is MCS contract which can min/burn token
    controller: AccountId,
    /// Owner is multisig contract which can preform upgrade
    owner: AccountId,
    token: FungibleToken,
    name: String,
    symbol: String,
    reference: String,
    reference_hash: Base64VecU8,
    decimals: u8,
    icon: Option<String>,
}

#[near_bindgen]
impl MCSToken {
    #[init]
    pub fn new(controller: AccountId, owner: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            controller,
            owner,
            token: FungibleToken::new(b"t".to_vec()),
            name: String::default(),
            symbol: String::default(),
            reference: String::default(),
            reference_hash: Base64VecU8(vec![]),
            decimals: 0,
            icon: None,
        }
    }

    pub fn set_metadata(
        &mut self,
        name: Option<String>,
        symbol: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: Option<u8>,
        icon: Option<String>,
    ) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        name.map(|name| self.name = name);
        symbol.map(|symbol| self.symbol = symbol);
        reference.map(|reference| self.reference = reference);
        reference_hash.map(|reference_hash| self.reference_hash = reference_hash);
        decimals.map(|decimals| self.decimals = decimals);
        icon.map(|icon| self.icon = Some(icon));
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

    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.controller,
            "Only controller can call burn"
        );

        self.token.internal_withdraw(&account_id, amount.into());
    }

    pub fn account_storage_usage(&self) -> StorageUsage {
        self.token.account_storage_usage
    }

    pub fn set_owner(&mut self, new_owner: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        self.owner = new_owner;
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn set_controller(&mut self, controller: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        self.controller = controller;
    }

    pub fn get_controller(&self) -> AccountId {
        self.controller.clone()
    }

    pub fn upgrade_self(&mut self, code: Base64VecU8) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        let current_id = env::current_account_id();
        let promise_id = env::promise_batch_create(&current_id);
        env::promise_batch_action_deploy_contract(promise_id, &code.0);
        env::promise_batch_action_function_call(
            promise_id,
            "migrate",
            &[],
            0,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY,
        );
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
            icon: self.icon.clone(),
            reference: Some(self.reference.clone()),
            reference_hash: Some(self.reference_hash.clone()),
            decimals: self.decimals,
        }
    }
}
