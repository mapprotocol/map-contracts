use crate::{CoreSwapMessage, TransferInParam};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{ext_contract, AccountId, CryptoHash, Promise, PromiseOrValue};

pub trait Transferable {
    fn get_transfer_in_param(&self) -> TransferInParam;
    fn basic_check(&self);
    fn get_to_chain(&self) -> U128;
    fn get_order_id(&self) -> CryptoHash;
    fn get_transfer_in_token(&self) -> AccountId;
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
    fn storage_balance_bounds(&self) -> StorageBalanceBounds;
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[ext_contract(ext_wnear_token)]
pub trait ExtWNearToken {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}

#[ext_contract(ext_butter_core)]
pub trait ExtButterCore {
    fn swap(&mut self, amount: U128, core_swap_msg: CoreSwapMessage) -> PromiseOrValue<U128>;
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
