use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, Promise, Gas};
use map_light_client::Validator;

const MAP_CLIENT_BINARY: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/map_light_client.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: Gas = Gas(200_000_000_000_000);

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct Factory {}

#[near_bindgen]
impl Factory {
    #[payable]
    pub fn create_map_client(
        &mut self,
        name: String,
        threshold: u64,
        validators: Vec<Validator>,
        epoch: u64,
        epoch_size: u64
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id());
        Promise::new(account_id.parse().unwrap())
            .create_account()
            .deploy_contract(MAP_CLIENT_BINARY.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "new".to_string(),
                json!({
                    "threshold": threshold,
                    "validators": validators,
                    "epoch": epoch,
                    "epoch_size": epoch_size })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }
}
