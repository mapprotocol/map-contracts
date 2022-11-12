use std::collections::HashSet;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::{ AccountId, Balance, PanicOnDefault, CryptoHash};
use admin_controlled::Mask;
use crate::prover::*;
use crate::{ChainType, MapCrossChainService};

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldMapCrossChainService {
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
    /// Map of chain id and chain type
    pub chain_id_type_map: UnorderedMap<u128, ChainType>,
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<CryptoHash>,
    /// Account of the owner
    pub owner: AccountId,
    /// Balance required to register a new account in the MCSToken
    pub mcs_storage_transfer_in_required: Balance,
    // Wrap token for near
    pub wrapped_token: String,
    // Near chain id
    pub near_chain_id: u128,
    // Nonce to generate order id
    pub nonce: u128,
    /// Mask determining all paused functions
    paused: Mask,
}

impl OldMapCrossChainService {
    pub fn migrate(self) -> MapCrossChainService {
        MapCrossChainService {
            map_client_account: self.map_client_account,
            map_bridge_address: self.map_bridge_address,
            mcs_tokens: self.mcs_tokens,
            fungible_tokens: self.fungible_tokens,
            fungible_tokens_storage_balance: self.fungible_tokens_storage_balance,
            token_decimals: self.token_decimals,
            native_to_chains: self.native_to_chains,
            chain_id_type_map: self.chain_id_type_map,
            used_events: self.used_events,
            owner: self.owner,
            mcs_storage_transfer_in_required: self.mcs_storage_transfer_in_required,
            wrapped_token: self.wrapped_token,
            near_chain_id: self.near_chain_id,
            map_chain_id: 0,
            nonce: self.nonce,
            paused: self.paused,
        }
    }
}