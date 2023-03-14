use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::Balance;
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise};

const MCS_BINARY: &'static [u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/mos.wasm");

const MCS_TOKEN_BINARY: &'static [u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/mos_token.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: Gas = Gas(200_000_000_000_000);

/// Initial balance for the MCSToken contract to cover storage and related.
const MCS_TOKEN_INIT_BALANCE: Balance = 5_000_000_000_000_000_000_000_000; // 5e24yN, 5N

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct Factory {}

#[near_bindgen]
impl Factory {
    #[payable]
    pub fn create_mos(
        &mut self,
        name: String,
        owner: AccountId,
        map_light_client: String,
        map_bridge_address: String,
        wrapped_token: AccountId,
        near_chain_id: U128,
        map_chain_id: U128,
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id());
        Promise::new(account_id.parse().unwrap())
            .create_account()
            .deploy_contract(MCS_BINARY.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "init".to_string(),
                json!({ "owner": owner, "map_light_client": map_light_client,
                    "map_bridge_address": map_bridge_address, "wrapped_token": wrapped_token,
                    "near_chain_id": near_chain_id, "map_chain_id": map_chain_id,})
                .to_string()
                .as_bytes()
                .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }

    #[payable]
    pub fn deploy_mcs_token(
        &mut self,
        name: String,
        controller: AccountId,
        owner: AccountId,
    ) -> Promise {
        assert!(
            env::attached_deposit() >= MCS_TOKEN_INIT_BALANCE,
            "Not enough attached deposit to complete mcs token creation"
        );

        let account_id = format!("{}.{}", name, env::current_account_id());

        Promise::new(account_id.parse().unwrap())
            .create_account()
            .transfer(MCS_TOKEN_INIT_BALANCE)
            .deploy_contract(MCS_TOKEN_BINARY.to_vec())
            .function_call(
                "new".to_string(),
                json!({ "controller": controller, "owner": owner })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }
}
