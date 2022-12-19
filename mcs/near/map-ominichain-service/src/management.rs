use crate::management::ChainType::{EvmChain, Unknown};
use crate::*;

const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(15_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ChainType {
    EvmChain,
    Unknown,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MAPOServiceV1 {
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
    // MAP chain id
    pub map_chain_id: u128,
    // Nonce to generate order id
    pub nonce: u128,
    /// Mask determining all paused functions
    pub paused: Mask,
}

#[near_bindgen]
impl MAPOServiceV2 {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_mcs: MAPOServiceV1 = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        MAPOServiceV2::from(old_mcs)
    }
    pub fn set_chain_type(&mut self, chain_id: U128, chain_type: ChainType) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );

        self.chain_id_type_map.insert(&chain_id.into(), &chain_type);
    }

    pub fn get_chain_type(&self, chain_id: U128) -> ChainType {
        let chain_id = chain_id.into();
        if chain_id == self.map_chain_id {
            return EvmChain;
        }
        let option = self.chain_id_type_map.get(&chain_id);
        if let Some(chain_type) = option {
            chain_type
        } else {
            Unknown
        }
    }

    pub fn set_owner(&mut self, new_owner: AccountId) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        self.owner = new_owner;
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn set_map_light_client(&mut self, map_client_account: AccountId) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.is_paused(PAUSE_TRANSFER_IN),
            "transfer in should be paused when setting map light client account"
        );

        self.map_client_account = map_client_account;
    }

    pub fn get_map_light_client(&self) -> AccountId {
        self.map_client_account.clone()
    }

    pub fn set_near_chain_id(&mut self, near_chain_id: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.is_paused(PAUSE_TRANSFER_OUT_TOKEN)
                && self.is_paused(PAUSE_TRANSFER_OUT_NATIVE)
                && self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN)
                && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
            "transfer/deposit out should be paused when setting near chain id"
        );

        self.near_chain_id = near_chain_id.into();
    }

    pub fn get_near_chain_id(&self) -> U128 {
        self.near_chain_id.into()
    }

    pub fn set_map_chain_id(&mut self, map_chain_id: U128) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN) && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
            "deposit out should be paused when setting map chain id"
        );

        self.map_chain_id = map_chain_id.into();
    }

    pub fn get_map_chain_id(&self) -> U128 {
        self.map_chain_id.into()
    }

    pub fn set_map_relay_address(&mut self, map_relay_address: String) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.is_paused(PAUSE_TRANSFER_IN),
            "transfer in should be paused when setting near chain id"
        );

        self.map_bridge_address = validate_eth_address(map_relay_address);
    }

    pub fn get_map_relay_address(&self) -> String {
        hex::encode(self.map_bridge_address)
    }

    pub fn upgrade_self(&mut self, code: Base64VecU8) {
        assert!(
            self.is_owner(),
            "unexpected caller {}",
            env::predecessor_account_id()
        );
        assert!(
            self.is_paused(PAUSE_DEPLOY_TOKEN)
                && self.is_paused(PAUSE_TRANSFER_IN)
                && self.is_paused(PAUSE_TRANSFER_OUT_TOKEN)
                && self.is_paused(PAUSE_TRANSFER_OUT_NATIVE)
                && self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN)
                && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
            "everything should be paused when upgrading mcs contract"
        );

        let current_id = env::current_account_id();
        let promise_id = env::promise_batch_create(&current_id);
        env::promise_batch_action_deploy_contract(promise_id, &code.0);
        env::promise_batch_action_function_call(
            promise_id,
            "migrate",
            &[],
            NO_DEPOSIT,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY,
        );
    }
}

impl MAPOServiceV2 {
    fn from(mos: MAPOServiceV1) -> Self {
        let mut registered_tokens = UnorderedMap::new(b"r".to_vec());
        for token in mos.mcs_tokens.keys() {
            registered_tokens.insert(&token.parse().unwrap(), &true);
        }
        for token in mos.fungible_tokens.keys() {
            registered_tokens.insert(&token.parse().unwrap(), &true);
        }
        Self {
            map_client_account: mos.map_client_account,
            map_bridge_address: mos.map_bridge_address,
            mcs_tokens: mos.mcs_tokens,
            fungible_tokens: mos.fungible_tokens,
            fungible_tokens_storage_balance: mos.fungible_tokens_storage_balance,
            token_decimals: mos.token_decimals,
            native_to_chains: mos.native_to_chains,
            chain_id_type_map: mos.chain_id_type_map,
            used_events: mos.used_events,
            owner: mos.owner,
            mcs_storage_balance_min: mos.mcs_storage_transfer_in_required,
            wrapped_token: mos.wrapped_token.parse().unwrap(),
            near_chain_id: mos.near_chain_id,
            map_chain_id: mos.map_chain_id,
            nonce: mos.nonce,
            paused: mos.paused,
            registered_tokens,
            ref_exchange: "ref-finance-101.testnet".to_string().parse().unwrap(),
            core_idle: vec![],
            core_total: vec![],
            amount_out: Default::default(),
            lost_found: UnorderedMap::new(b"l".to_vec()),
        }
    }
}
