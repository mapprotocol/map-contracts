use crate::*;
use crate::management::ChainType::{EvmChain, Unknown};


const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(15_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ChainType {
    EvmChain,
    Unknown,
}

#[near_bindgen]
impl MapCrossChainService{
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let mcs: MapCrossChainService = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        mcs
    }
    pub fn set_chain_type(&mut self, chain_id: U128, chain_type: ChainType) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());

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
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        self.owner = new_owner;
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn set_map_light_client(&mut self, map_client_account: AccountId) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.is_paused(PAUSE_TRANSFER_IN),
                "transfer in should be paused when setting map light client account");

        self.map_client_account = map_client_account;
    }

    pub fn get_map_light_client(&self) -> AccountId {
        self.map_client_account.clone()
    }

    pub fn set_near_chain_id(&mut self, near_chain_id: U128) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.is_paused(PAUSE_TRANSFER_OUT_TOKEN)
                    && self.is_paused(PAUSE_TRANSFER_OUT_NATIVE)
                    && self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN)
                    && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
                "transfer/deposit out should be paused when setting near chain id");

        self.near_chain_id = near_chain_id.into();
    }

    pub fn get_near_chain_id(&self) -> U128 {
        self.near_chain_id.into()
    }

    pub fn set_map_chain_id(&mut self, map_chain_id: U128) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN)
                    && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
                "deposit out should be paused when setting map chain id");

        self.map_chain_id = map_chain_id.into();
    }

    pub fn get_map_chain_id(&self) -> U128 {
        self.map_chain_id.into()
    }

    pub fn set_map_relay_address(&mut self, map_relay_address: String) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.is_paused(PAUSE_TRANSFER_IN),
                "transfer in should be paused when setting near chain id");

        self.map_bridge_address = validate_eth_address(map_relay_address);
    }

    pub fn get_map_relay_address(&self) -> String {
        hex::encode(self.map_bridge_address)
    }

    pub fn upgrade_self(&mut self, code: Base64VecU8) {
        assert!(self.is_owner(), "unexpected caller {}", env::predecessor_account_id());
        assert!(self.is_paused(PAUSE_DEPLOY_TOKEN)
                    && self.is_paused(PAUSE_TRANSFER_IN)
                    && self.is_paused(PAUSE_TRANSFER_OUT_TOKEN)
                    && self.is_paused(PAUSE_TRANSFER_OUT_NATIVE)
                    && self.is_paused(PAUSE_DEPOSIT_OUT_TOKEN)
                    && self.is_paused(PAUSE_DEPOSIT_OUT_NATIVE),
                "everything should be paused when upgrading mcs contract");

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