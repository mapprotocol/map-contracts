use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::PublicKey;
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise};

const MULTISIG_BINARY: &'static [u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/multisig.wasm");

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
        num_confirmations: u32,
        request_lock: U64,
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
}
