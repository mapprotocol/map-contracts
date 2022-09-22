use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::PublicKey;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, AccountId, Promise, Gas};

const MULTISIG_BINARY: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/multisig.wasm");
const MCS_BINARY: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/mcs.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: Gas = Gas(200_000_000_000_000);

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum MultisigMember {
    AccessKey { public_key: PublicKey },
    Account { account_id: AccountId },
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct Factory {}

#[near_bindgen]
impl Factory {
    #[payable]
    pub fn create_multisig(
        &mut self,
        name: String,
        members: Vec<MultisigMember>,
        num_confirmations: u64,
        request_lock: u64,
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id());
        Promise::new(account_id.parse().unwrap())
            .create_account()
            .deploy_contract(MULTISIG_BINARY.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "new".to_string(),
                json!({ "members": members,
                    "num_confirmations": num_confirmations,
                    "request_lock": request_lock})
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }

    #[payable]
    pub fn create_mcs(
        &mut self,
        name: String,
        owner: AccountId,
        map_light_client: String,
        map_bridge_address: String,
        wrapped_token: String,
        near_chain_id: u64,
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
                    "near_chain_id": near_chain_id })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }
}
