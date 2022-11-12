use std::collections::HashSet;
use std::fmt::format;
use std::fs;
use std::ops::Index;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use hex;
use near_sdk::json_types::U128;
use near_sdk::{Balance, log, serde};
use near_sdk::serde::{Serialize, Deserialize};
// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::{prelude::*, Worker, Contract, AccountId, Account, Network, DevNetwork};
use workspaces::network::{Sandbox, Testnet};
use workspaces::operations::CallTransaction;
use workspaces::result::CallExecutionDetails;
use workspaces::types::{KeyType, SecretKey};
use map_light_client::{EpochRecord, Validator};

const MOCK_MAP_CLIENT_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mock_map_client.wasm";
const MULTISIG_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/multisig.wasm";
const MCS_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs.wasm";
const MCS_TOKEN_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs_token.wasm";
const WNEAR_WASM_FILEPATH: &str = "./tests/data/w_near.wasm";
const NEAR_SANDBOX_BIN_PATH: &str = "NEAR_SANDBOX_BIN_PATH";
const MAP_BRIDGE_ADDRESS: &str = "765a5a86411ab8627516cbb77d5db00b74fe610d";
const MAP_CHAIN_ID: u128 = 22776;
const DEV_ACCOUNT_SEED: &str = "testificate";
const TRANSFER_OUT_TYPE: &str = "2ef1cdf83614a69568ed2c96a275dd7fb2e63a464aa3a0ffe79f55d538c8b3b5";
const DEPOSIT_OUT_TYPE: &str = "150bd848adaf4e3e699dcac82d75f111c078ce893375373593cc1b9208998377";

const ORDER_ID_DEPOSIT: Balance = 1640000000000000000000;


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMsg {
    pub msg_type: u8,
    // 0: Transfer or 1: Deposit
    pub to: Vec<u8>,
    pub to_chain: u128, // if msg_type is 1, it is omitted
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ChainType {
    EvmChain,
    Unknown,
}

#[tokio::test]
async fn test_multisig_access_key_methods() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let receiver = worker.dev_create_account().await?;
    let sk0 = SecretKey::from_seed(KeyType::ED25519, "sk0");
    let sk1 = SecretKey::from_seed(KeyType::ED25519, "sk1");

    let init_value = r#"{
        "members": [{ "public_key": ""}],
        "num_confirmations": 1,
        "request_lock": 5000000000
    }"#;
    let mut init_args: serde_json::Value = serde_json::from_str(init_value).unwrap();
    init_args["members"][0]["public_key"] = json!(sk0.public_key());

    let multisig = deploy_and_init_multisig(&worker, init_args).await?;
    let member = new_account(multisig.id(), &sk0);
    println!("account: {:?}", member.id());

    let request_value = r#"{
        "receiver_id": "",
        "actions": [{"type": "Transfer", "amount": "1000"}]
    }"#;
    let mut request: serde_json::Value = serde_json::from_str(request_value).unwrap();
    request["receiver_id"] = json!(receiver.id());
    let request_id = gen_call_transaction_by_account(&worker, &member, &multisig, "add_request", json!({"request": request}), false)
        .transact()
        .await?
        .json::<u32>()?;

    let res = gen_call_transaction_by_account(&worker, &member, &multisig, "confirm", json!({"request_id": request_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "confirm should succeed");

    worker.fast_forward(1000).await?;

    let res = gen_call_transaction_by_account(&worker, &member, &multisig, "delete_request", json!({"request_id": request_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "delete_request should succeed");

    let request_id = gen_call_transaction_by_account(&worker, &member, &multisig, "add_request_and_confirm", json!({"request": request}), false)
        .transact()
        .await?
        .json::<u32>()?;
    assert!(res.is_success(), "add_request_and_confirm should succeed");

    worker.fast_forward(1000).await?;

    let res = gen_call_transaction_by_account(&worker, &member, &multisig, "execute", json!({"request_id": request_id}), false)
        .transact()
        .await;
    assert!(res.is_err(), "call execute should fail");
    println!("err: {}", res.as_ref().err().unwrap());
    assert!(res.err().unwrap().to_string().contains("InvalidAccessKeyError"));

    Ok(())
}

#[tokio::test]
async fn test_deploy_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let mut token_names: HashSet<String> = HashSet::default();
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        let token_account = format!("{}.{}", token_name, mcs.id().to_string());
        token_names.insert(token_account.clone());
        let res = mcs
            .call(&worker, "deploy_mcs_token")
            .args_json(json!({"address": token_name}))?
            .gas(300_000_000_000_000)
            .deposit(parse_near!("10 N"))
            .transact()
            .await?;
        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);
    }

    let tokens = mcs
        .call(&worker, "get_mcs_tokens")
        .view()
        .await?
        .json::<Vec<(String, HashSet<u128>)>>()?;

    println!("tokens:{:?}", tokens);

    assert_eq!(2, tokens.len(), "wrong mcs tokens size");
    for i in 0..2 {
        let token = tokens.get(i).unwrap();
        assert_eq!(0, token.1.len());
        assert!(token_names.remove(&token.0));
    }

    assert_eq!(0, token_names.len());

    Ok(())
}

#[tokio::test]
async fn test_owner() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;

    let account_id: AccountId = "mcs.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let owner = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let mcs = worker.dev_deploy(&std::fs::read(MCS_WASM_FILEPATH)?).await?;
    println!("deploy mcs contract id: {:?}", mcs.id());

    let res = mcs
        .call(&worker, "init")
        .args_json(json!({"owner": owner.id(),
            "map_light_client": "map_light_client.near",
            "map_bridge_address":MAP_BRIDGE_ADDRESS.to_string(),
            "wrapped_token": wnear.id().to_string(),
            "near_chain_id": "1313161555",
            "map_chain_id": "22776",
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    let chain_id = 100;
    let token_name = "mcs_token";
    let token_account = format!("{}.{}", token_name, mcs.id().to_string());

    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account, "to_chain": chain_id}), false)
        .transact()
        .await;
    assert!(res.is_err(), "add_mcs_token_to_chain should be called by owner");

    let res = gen_call_transaction_by_account(&worker, &owner, &mcs, "add_mcs_token_to_chain", json!({"token": token_account, "to_chain": chain_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": chain_id, "chain_type": "EvmChain"}), false)
        .transact()
        .await;
    assert!(res.is_err(), "set_chain_type should be called by owner");

    let res = gen_call_transaction_by_account(&worker, &owner, &mcs, "set_chain_type", json!({"chain_id": chain_id, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let ft = deploy_and_init_ft(&worker).await?;
    let token_name0 = ft.id().to_string();
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_name0, "to_chain": chain_id}), false)
        .transact()
        .await;
    assert!(res.is_err(), "add_fungible_token_to_chain should be called by owner");

    let res = gen_call_transaction_by_account(&worker, &owner, &mcs, "add_fungible_token_to_chain", json!({"token": token_name0, "to_chain": chain_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": chain_id}), false)
        .transact()
        .await;
    assert!(res.is_err(), "add_native_to_chain should be called by owner");

    let res = gen_call_transaction_by_account(&worker, &owner, &mcs, "add_native_to_chain", json!({"to_chain": chain_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    Ok(())
}

#[tokio::test]
async fn test_manage_to_chain_type() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let chain_id = 100;
    let chain_type = ChainType::EvmChain;

    let ret = gen_call_transaction(&worker, &mcs, "get_chain_type", json!({"chain_id": chain_id}), false)
        .view()
        .await?
        .json::<ChainType>()?;
    assert_eq!(ChainType::Unknown, ret, "chain type should be unknonw");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": chain_id, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let ret = gen_call_transaction(&worker, &mcs, "get_chain_type", json!({"chain_id": chain_id}), false)
        .view()
        .await?
        .json::<ChainType>()?;
    assert_eq!(chain_type, ret, "chain type should be set");

    Ok(())
}

#[tokio::test]
async fn test_manage_near_chain_id() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let near_chain_id = "5566818579631833088";
    let paused_mask = 1 << 2 | 1 << 3 | 1 << 4 | 1 << 5;

    let ret = gen_call_transaction(&worker, &mcs, "get_near_chain_id", json!({}), false)
        .view()
        .await?
        .json::<u128>()?;

    println!("get_near_chain_id {}", ret);
    assert_eq!(1313161555, ret, "get default near chain id");

    let res = gen_call_transaction(&worker, &mcs, "set_near_chain_id", json!({"near_chain_id": near_chain_id}), false)
        .transact()
        .await;
    assert!(res.is_err(), "set_chain_type should fail because of not paused");

    let res = gen_call_transaction(&worker, &mcs, "set_paused", json!({"paused": paused_mask}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_paused should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_near_chain_id", json!({"near_chain_id": near_chain_id}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let ret = gen_call_transaction(&worker, &mcs, "get_near_chain_id", json!({}), false)
        .view()
        .await?
        .json::<u128>()?;
    println!("get near chain id {}", ret);
    let s = format!("{}", ret);
    assert_eq!(near_chain_id, s, "near chain id should be set");

    println!("max u128 {}", u128::MAX);

    Ok(())
}

#[tokio::test]
async fn test_manage_map_relay_address() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let map_relay_address = "aaaa5a86411ab8627516cbb77d5db00b74fe610d";
    let paused_mask = 1 << 1;

    let ret = gen_call_transaction(&worker, &mcs, "get_map_relay_address", json!({}), false)
        .view()
        .await?
        .json::<String>()?;
    assert_eq!(MAP_BRIDGE_ADDRESS.to_string(), ret, "get default map relay address");

    let res = gen_call_transaction(&worker, &mcs, "set_map_relay_address", json!({"map_relay_address": map_relay_address}), false)
        .transact()
        .await;
    assert!(res.is_err(), "set_map_relay_address should fail because of not paused");

    let res = gen_call_transaction(&worker, &mcs, "set_paused", json!({"paused": paused_mask}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_paused should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_map_relay_address", json!({"map_relay_address": map_relay_address}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_map_relay_address should succeed");

    let ret = gen_call_transaction(&worker, &mcs, "get_map_relay_address", json!({}), false)
        .view()
        .await?
        .json::<String>()?;
    println!("get map relay address {}", ret);
    assert_eq!(map_relay_address.to_string(), ret, "map relay address should be set");

    let res = gen_call_transaction(&worker, &mcs, "set_map_relay_address", json!({"map_relay_address": "aaaa5a86411ab8627516cbb77d5db00b74f"}), false)
        .transact()
        .await;
    assert!(res.is_err(), "set_map_relay_address should fail because of invalid eth address");

    Ok(())
}

#[tokio::test]
async fn test_manage_mcs_token_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        let token_account = format!("{}.{}", token_name, mcs.id().to_string());

        let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account, "to_chain": i}), false)
            .transact()
            .await;
        assert!(res.is_err(), "add_mcs_token_to_chain should fail since it is not deployed");

        let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
            .transact()
            .await?;
        assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);

        let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "add_mcs_token_to_chain should succeed since it has been deployed");

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_account, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(is_valid, "mcs token {} to chain {} should be valid", token_account, i);

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_account, "to_chain": i+1}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "mcs token {} to chain {} should be invalid", token_account, i + 1);

        let res = gen_call_transaction(&worker, &mcs, "remove_mcs_token_to_chain", json!({"token": token_account, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "remove_mcs_token_to_chain should succeed");

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_account, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "mcs token {} to chain {} should be invalid", token_account, i);
    }

    Ok(())
}

#[tokio::test]
async fn test_manage_fungible_token_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let mut token_name0 = "eth_token".to_string();
    let to_chain = 1;
    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name0, "to_chain": to_chain}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name0, to_chain);

    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_name0, "to_chain": to_chain}), false)
        .transact()
        .await;
    assert!(res.is_err(), "add_fungible_token_to_chain should fail since the ft token does not exist");

    let ft = deploy_and_init_ft(&worker).await?;
    token_name0 = ft.id().to_string();
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_name0, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed since it has been deployed");

    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name0, "to_chain": to_chain}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(is_valid, "fungible token {} to chain {} should be valid", token_name0, to_chain);

    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name0, "to_chain": to_chain+1}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name0, to_chain + 1);

    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_name0, "to_chain": to_chain + 1}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name0, "to_chain": to_chain+1}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(is_valid, "fungible token {} to chain {} should be valid", token_name0, to_chain + 1);

    let ft = deploy_and_init_ft(&worker).await?;
    let token_name1 = ft.id().to_string();
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_name1, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed since it has been deployed");

    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name1, "to_chain": to_chain}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(is_valid, "fungible token {} to chain {} should be valid", token_name1, to_chain);

    let tokens = mcs
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<(String, HashSet<u128>)>>()?;
    assert_eq!(2, tokens.len(), "wrong fungible tokens size");
    assert_eq!(token_name0, tokens.get(0).unwrap().0, "{} is not contained", token_name0);
    assert_eq!(token_name1, tokens.get(1).unwrap().0, "{} is not contained", token_name1);

    let res = gen_call_transaction(&worker, &mcs, "remove_fungible_token_to_chain", json!({"token": token_name1, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "remove_fungible_token_to_chain should succeed");

    let is_valid = gen_call_transaction(&worker, &mcs, "valid_fungible_token_out", json!({"token": token_name1, "to_chain": to_chain}), false)
        .view()
        .await?
        .json::<bool>()?;
    assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name1, to_chain);

    let tokens = mcs
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<(String, HashSet<u128>)>>()?;
    assert_eq!(1, tokens.len(), "wrong fungible tokens size");
    assert_eq!(token_name0, tokens.get(0).unwrap().0, "{} is not contained", token_name0);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.testnet".parse().unwrap();
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;

    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_wrong_bridge() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("unexpected map mcs address"), "should be mcs address error");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_no_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("is not mcs token or fungible token or empty"), "token is invalid");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_no_light_client() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("verify proof failed"), "should be cross contract call error");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT - 1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("not enough deposit for record proof"), "should be deposit error");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("not enough deposit for mcs token mint"), "should be deposit error");
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 > dev_balance_0 - dev_balance_1);
    assert!(dev_balance_1 - dev_balance_2 < dev_balance_0 - dev_balance_1 + ORDER_ID_DEPOSIT / 2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let to: AccountId = "pandarr.testnet".parse().unwrap();
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT + 1250000000000000000000)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_3 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_3);
    println!("dev_account decrease {}", dev_balance_2 - dev_balance_3); //35777116013107000000000
    assert!(dev_balance_2 - dev_balance_3 > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1250000000000000000000);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());

    let to: AccountId = "pandarr.testnet".parse().unwrap();
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    let balance_0 = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance_0.0, "balance of {} is incorrect", to);

    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("is used"), "transfer in should failed because of used proof");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let balance_1 = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(balance_0.0, balance_1.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let ft = deploy_and_init_ft_with_account(&worker, &account).await?;
    let to_chain = 1;

    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_account, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(1000);
    let res = ft.call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(900, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_no_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let ft = deploy_and_init_ft_with_account(&worker, &account).await?;

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(1000);
    let res = ft.call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.err());
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of mcs is incorrect");

    let to_chain = 1;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_account, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - 100, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let ft = deploy_and_init_ft_with_account(&worker, &account).await?;
    let to_chain = 1;

    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_account, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let total: U128 = U128::from(1000);
    let res = ft.call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT - 1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("not enough deposit for record proof"), "should be deposit error");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("not enough deposit for ft transfer"), "should be deposit error");
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 > dev_balance_0 - dev_balance_1);
    assert!(dev_balance_1 - dev_balance_2 < dev_balance_0 - dev_balance_1 + ORDER_ID_DEPOSIT / 2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT + parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_3 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_3);
    println!("dev_account decrease {}", dev_balance_2 - dev_balance_3); //35777116013107000000000
    assert!(dev_balance_2 - dev_balance_3 > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT);
    assert!(dev_balance_2 - dev_balance_3 < dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + parse_near!("1 N"));

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - 100, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_not_enough_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let ft = deploy_and_init_ft_with_account(&worker, &account).await?;
    let to_chain = 1;

    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": token_account, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(10);
    let res = ft.call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("transfer in token failed"), "should be transfer in token error");
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0, balance.0, "balance of mcs is incorrect");

    let total1: U128 = U128::from(1000);
    let res = ft.call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total1}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer in 2: account {} balance: {}", dev_account.id(), dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account.call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 + total1.0 - 100, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in: account {} balance: {}", account_id, balance_0);
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should success");
    println!("logs {:?}", res.logs());
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_1);
    assert_eq!(amount, balance_1 - balance_0, "should transfer in 100 yocto near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(amount, mcs_wnear_0.0 - mcs_wnear_1.0);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in: account {} balance: {}", account_id, balance_0);
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(100)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.err());
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_1);
    assert_eq!(balance_0, balance_0, "to account balance should not change");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(mcs_wnear_0.0, mcs_wnear_1.0, "wnear balance of mcs should not change");

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.err());
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    assert!(dev_balance_1 - dev_balance_2 > dev_balance_0 - dev_balance_1);
    assert!(dev_balance_1 - dev_balance_2 < dev_balance_0 - dev_balance_1 + ORDER_ID_DEPOSIT / 2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT + 1)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());

    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in 3: account {} balance: {}", account_id, balance_2);
    assert_eq!(balance_1 + amount, balance_2, "should transfer in {} yocto near", amount);
    let dev_balance_3 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in 3: account {} balance: {}", dev_account.id(), dev_balance_3);
    println!("dev account balance decrease: {}", dev_balance_1 - dev_balance_3);
    assert!(dev_balance_2 - dev_balance_3 > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1);
    assert!(dev_balance_2 - dev_balance_3 < dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1 + parse_near!("1 N"));

    let mcs_wnear_2 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_2.0);
    assert_eq!(mcs_wnear_0.0 - amount, mcs_wnear_2.0, "wnear balance of mcs should decrease {}", amount);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_wnear() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(99)
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);
    assert!(mcs_wnear_0.0 < amount, "should less than expected amount");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in: account {} balance: {}", account_id, balance_0);

    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.err());
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_1);
    assert_eq!(balance_0, balance_1, "account should get 0 near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(mcs_wnear_0.0, mcs_wnear_1.0, "wnear for mcs should not change");

    // deposit to wnear and transfer in again
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(100)
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should success");
    println!("log {:?}", res.logs());

    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in 2: account {} balance: {}", account_id, balance_2);
    assert_eq!(balance_1 + amount, balance_2, "account should get {} yocto near", amount);
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in 2: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("dev account balance decrease: {}", dev_balance_1 - dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in 2: account {} wnear balance: {}", mcs.id(), mcs_wnear_2.0);
    assert_eq!(mcs_wnear_1.0 + 100 - amount, mcs_wnear_2.0, "wnear for mcs should decrease {} tocto near", amount);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_no_to_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);

    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.err());
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(mcs_wnear_0.0, mcs_wnear_1.0, "wnear of mcs should not change");

    // create account and transfer in again should succeed
    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in 2: account {} balance: {}", account_id, balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should success");
    println!("logs {:?}", res.logs());
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in 2: account {} balance: {}", account_id, balance_1);
    assert_eq!(amount, balance_1 - balance_0, "should transfer in 100 yocto near");
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in 2: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("dev account balance decrease: {}", dev_balance_1 - dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in 2: account {} wnear balance: {}", mcs.id(), mcs_wnear_2.0);
    assert_eq!(mcs_wnear_1.0 - amount, mcs_wnear_2.0, "wnear of mcs should decrease {} yocto near", amount);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in: account {} balance: {}", account_id, balance_0);
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should success");
    println!("logs {:?}", res.logs());
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_1);
    assert_eq!(amount, balance_1 - balance_0, "should transfer in 100 yocto near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(amount, mcs_wnear_0.0 - mcs_wnear_1.0);

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("err {:?}", res.err());
    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in 2: account {} balance: {}", account_id, balance_2);
    assert_eq!(balance_1, balance_2, "to account balance should not change");
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in 2: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("dev account balance decrease: {}", dev_balance_1 - dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in 2: account {} wnear balance: {}", mcs.id(), mcs_wnear_2.0);
    assert_eq!(mcs_wnear_1.0, mcs_wnear_2.0, "wnear balance of mcs should not change");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_gas() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs.as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_0.0);

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!("before transfer in: account {} balance: {}", account_id, balance_0);
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_balance_0);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(140_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("Exceeded the prepaid gas"), "should be gas error");
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_1);
    assert_eq!(balance_1, balance_0, "should transfer in 0 yocto near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_1);
    println!("dev account balance decrease: {}", dev_balance_0 - dev_balance_1);
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(mcs_wnear_0.0, mcs_wnear_1.0);

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(180_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!("after transfer in: account {} balance: {}", account_id, balance_2);
    assert_eq!(amount, balance_2 - balance_1, "should transfer in {} yocto near", amount);
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("dev account balance decrease: {}", dev_balance_1 - dev_balance_2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account.call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer in: account {} wnear balance: {}", mcs.id(), mcs_wnear_1.0);
    assert_eq!(amount, mcs_wnear_1.0 - mcs_wnear_2.0);
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res.logs());
    assert!(res.logs().get(1).unwrap().contains(TRANSFER_OUT_TYPE), "can not get transfer out log");
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - amount.0, balance.0, "balance of {} is incorrect", dev_account.id());

    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());
    assert_ne!(res.logs().get(1).unwrap(), res1.logs().get(1).unwrap(), "log should be different");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_invalid_to_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F945").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("address length is incorrect for evm chain type"), "should be address error");
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("amount too small"), "should be amount too small error");
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let amount: U128 = U128(1000000000000000000000);
    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 18).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(1000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("amount too small"), "should be amount too small error");
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let amount: U128 = U128(1000000000000000);
    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_with_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.err());
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    assert!(mcs_balance_2 - mcs_balance_1 < parse_near!("1 N"));
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0, balance.0, "balance of {} is incorrect", dev_account.id());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let to_chain: u64 = 1000;
    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!("before mint: account {} balance: {}", mcs.id(), mcs_balance_0);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!("after mint: account {} balance: {}", mcs.id(), mcs_balance_1);
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!("before transfer out: account {} balance: {}", dev_account.id(), dev_balance_1);
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("is not supported"), "should be to chain not support error");
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", mcs.id(), mcs_balance_2);
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!("after transfer out: account {} balance: {}", dev_account.id(), dev_balance_2);
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!("{}, {}", (dev_balance_1 - dev_balance_2) * 3 / 10, mcs_balance_2 - mcs_balance_1);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0, balance.0 as u128, "balance of {} is incorrect", dev_account.id());

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_burn_failed() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let dev_account = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    println!("before mint: account {} balance: {}", mcs.id(), mcs.view_account(&worker).await?.balance);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("after mint: account {} balance: {}", mcs.id(), mcs.view_account(&worker).await?.balance);
    assert!(res.is_success(), "mint should succeed");

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: U128 = U128(1000000000000000000000000000 + 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error: {}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("get failed result from cross contract"), "should be cross contract call error");

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of {} is incorrect", dev_account.id());

    let amount: U128 = U128(1000000000000000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_token should succeed");
    println!("logs: {:?}", res.logs());

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(1, balance.0, "balance of {} is incorrect", dev_account.id());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(3, res.logs().len(), "should have 3 logs");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of root account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0 as u128, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_invalid_to_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = "abc".as_bytes().to_vec();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount, balance, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft_with_decimal(&worker, 18).await?;

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(1000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount, balance, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_wrong_type() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 2,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of root account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 100;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let to_chain: u64 = 1000;
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_not_registered() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1000;
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "ft_transfer_call should fail");
    println!("ft_transfer_call error: {:?}", res.err());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_native failed");
    println!("log: {:?}", res.logs());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(amount < balance_from_0 - balance_from_1, "sender's balance should decrease more than {}", amount);

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(amount, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_invalid_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("0.001 N");
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31").unwrap();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(balance_from_0 - balance_from_1 < parse_near!("1 N"), "sender's balance decrease too much");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("0.0009 N");
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(balance_from_0 - balance_from_1 < parse_near!("1 N"), "sender's balance decrease too much");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    let amount: u128 = parse_near!("0.001 N");
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_native should succeed");
    println!("logs: {:?}", res.logs());

    let balance_from_2 = from.view_account(&worker).await?.balance;
    println!("after transfer out native 2, account {} balance {}", from.id(), balance_from_2);
    assert!(balance_from_1 - balance_from_2 > amount, "sender's balance should decrease more than {}", amount);

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(amount, balance.0, "wnear balance of mcs contract account == transferred out native token amount");
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_no_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(0)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("amount should > 0"), "should be deposit error");

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(balance_from_0 > balance_from_1, "sender's balance should decrease more than 0");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    let amount = parse_near!("0.001 N");
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_native should succeed");
    println!("logs: {:?}", res.logs());

    let balance_from_2 = from.view_account(&worker).await?.balance;
    println!("after transfer out native 2, account {} balance {}", from.id(), balance_from_2);
    assert!(balance_from_0 - balance_from_1 > amount, "sender's balance should decrease more than 1");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(amount, balance.0, "wnear balance of mcs contract account == transferred out native token amount");
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(balance_from_0 - balance_from_1 < parse_near!("1 N"), "sender's balance decrease too much");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_not_enough_gas() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: u64 = 1000;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(&worker, &mcs, "set_chain_type", json!({"chain_id": to_chain, "chain_type": "EvmChain"}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!("before transfer out native, account {} balance {}", from.id(), balance_from_0);
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(30_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_1);
    assert!(balance_from_0 - balance_from_1 < parse_near!("1 N"), "sender's balance decrease too much");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(60_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_2 = from.view_account(&worker).await?.balance;
    println!("after transfer out native, account {} balance {}", from.id(), balance_from_2);
    assert!(balance_from_1 - balance_from_2 < parse_near!("1 N"), "sender's balance decrease too much");

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "wnear balance of mcs contract account == transferred out native token amount");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = parse_near!("10 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);
    log!("{}, {}", balance_from_0- balance_from_1, balance_mcs_1 - balance_mcs_0);
    assert!(balance_from_0 - balance_from_1 > amount);
    assert!(balance_mcs_1 - balance_mcs_0 > amount);
    assert!((balance_from_0 - balance_from_1 - amount) > (balance_mcs_1 - balance_mcs_0 - amount));
    log!("{}", balance_from_0 - balance_from_1 - amount);
    log!("{}", balance_mcs_1 - balance_mcs_0 - amount);

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native_invalid_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let to = hex::decode("abcd").unwrap();
    let amount: u128 = parse_near!("0.001 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("address length is incorrect for evm chain type"), "should be invalid address error");
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = parse_near!("0.0009 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("amount too small"), "should be amount too small error");
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);

    let amount: u128 = parse_near!("0.001 N");
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("logs: {:?}", res.logs());
    let balance_from_2 = from.view_account(&worker).await?.balance;
    let balance_mcs_2 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_2:{}, balance_from_1:{}", balance_from_2, balance_from_1);
    log!("balance_mcs_2:{}, balance_mcs_1:{}", balance_mcs_2, balance_mcs_1);
    log!("{}, {}", balance_from_1- balance_from_2, balance_mcs_2 - balance_mcs_1);
    assert!(balance_from_1 - balance_from_2 > amount);
    assert!(balance_mcs_2 - balance_mcs_1 > amount);
    assert!((balance_from_1 - balance_from_2 - amount) > (balance_mcs_2 - balance_mcs_1 - amount));
    log!("{}", balance_from_1 - balance_from_2 - amount);
    log!("{}", balance_mcs_2 - balance_mcs_1 - amount);

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native_no_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = 0;
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(res.err().unwrap().to_string().contains("amount should > 0"), "should be amount error");
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);
    log!("{}, {}", balance_from_0- balance_from_1, balance_mcs_1 - balance_mcs_0);
    assert!(balance_from_0 - balance_from_1 > amount);
    assert!(balance_mcs_1 - balance_mcs_0 > amount);
    assert!((balance_from_0 - balance_from_1 - amount) > (balance_mcs_1 - balance_mcs_0 - amount));
    log!("{}", balance_from_0 - balance_from_1 - amount);
    log!("{}", balance_mcs_1 - balance_mcs_0 - amount);

    let amount: u128 = parse_near!("1 N");
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("logs: {:?}", res.logs());
    assert!(res.logs().get(1).unwrap().contains(DEPOSIT_OUT_TYPE), "should be deposit out log");
    let balance_from_2 = from.view_account(&worker).await?.balance;
    let balance_mcs_2 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_2:{}, balance_from_1:{}", balance_from_2, balance_from_1);
    log!("balance_mcs_2:{}, balance_mcs_1:{}", balance_mcs_2, balance_mcs_1);
    log!("{}, {}", balance_from_1- balance_from_2, balance_mcs_2 - balance_mcs_1);
    assert!(balance_from_1 - balance_from_2 > amount);
    assert!(balance_mcs_2 - balance_mcs_1 > amount);
    assert!((balance_from_1 - balance_from_2 - amount) > (balance_mcs_2 - balance_mcs_1 - amount));
    log!("{}", balance_from_1 - balance_from_2 - amount);
    log!("{}", balance_mcs_2 - balance_mcs_1 - amount);

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = MAP_CHAIN_ID as _;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0 as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(3, res.logs().len(), "should have 3 logs");
    assert!(res.logs().get(2).unwrap().contains(DEPOSIT_OUT_TYPE), "should be deposit out log");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of root account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after deposit out ft balance of from account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_invalid_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = MAP_CHAIN_ID as u64;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount":total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("abcd").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = MAP_CHAIN_ID as u64;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount":total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft_with_decimal(&worker, 18).await?;

    let to_chain: u64 = MAP_CHAIN_ID as u64;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount":total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(1000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 100;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let to_chain: u64 = 1000;
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount":total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_not_registered() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    let ft = deploy_and_init_ft(&worker).await?;

    let to_chain: u64 = 1;
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": total}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 0,
        to,
        to_chain: to_chain as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "ft_transfer_call should fail");
    println!("ft_transfer_call error: {:?}", res.err());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let to_chain: u64 = MAP_CHAIN_ID as _;
    let token_name = "mcs_token_0";
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before deposit out mcs token balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0 as _,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(3, res.logs().len(), "should have 3 logs");
    assert!(res.logs().get(2).unwrap().contains(DEPOSIT_OUT_TYPE), "should be deposit out log");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of root account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after deposit out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let to_chain: u64 = MAP_CHAIN_ID as _;
    let token_name = "mcs_token_0";
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0,
    };
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let to_chain: u64 = MAP_CHAIN_ID as _;
    let token_name = "mcs_token_0";
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 18).await?;

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0,
    };
    let amount: U128 = U128(1000000000000000 - 1);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total.0 - amount.0, balance.0, "after transfer out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let to_chain: u64 = 100;
    let token_name = "mcs_token_0";
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
    deploy_mcs_token_and_set_decimals(&worker, &mcs, token_name.to_string(), 24).await?;

    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_account.to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs.as_account().call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of {} is {:?}", from.id(), balance);
    assert_eq!(total, balance, "before transfer out ft balance of root account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        msg_type: 1,
        to,
        to_chain: 0,
    };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, &token_account, "ft_transfer_call")
        .args_json((mcs.id().to_string(), amount, Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of from account is {:?}", balance);
    assert_eq!(total, balance, "after transfer out ft balance of from account");

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

fn gen_call_transaction<'a, U: serde::Serialize>(worker: &'a Worker<Sandbox>, contract: &'a Contract, function: &'a str, args: U, deposit: bool) -> CallTransaction<'a, 'a, impl DevNetwork> {
    let call_tx = contract
        .call(&worker, function)
        .args_json(args)
        .unwrap()
        .gas(300_000_000_000_000);
    if deposit {
        call_tx.deposit(7_000_000_000_000_000_000_000_000)
    } else {
        call_tx
    }
}

fn gen_call_transaction_by_account<'a, U: serde::Serialize>(worker: &'a Worker<Sandbox>, account: &Account, contract: &'a Contract, function: &'a str, args: U, deposit: bool) -> CallTransaction<'a, 'a, impl DevNetwork> {
    let call_tx = account
        .call(&worker, &contract.id(), function)
        .args_json(args)
        .unwrap()
        .gas(300_000_000_000_000);
    if deposit {
        call_tx.deposit(7_000_000_000_000_000_000_000_000)
    } else {
        call_tx
    }
}

fn new_account(account_id: &AccountId, sk: &SecretKey) -> Account {
    let account_value = r#"{
    "account_id": "",
    "public_key": "ed25519:5WMgq6gKZbAr7xBZmXJHjnj4C3UZkNJ4F5odisUBFcRh",
    "secret_key": "ed25519:3KyUucv7xFhA7xcjvS8owYeTotN2zYPc8AWhcRDkMG9ejac4gQsdVqDRrhh1v22ccuSK1JEFkhL7mzoKSuHKVyBH"
}"#;

    let mut account_json: serde_json::Value = serde_json::from_str(account_value).unwrap();
    account_json["account_id"] = json!(account_id);
    account_json["public_key"] = json!(sk.public_key());
    account_json["secret_key"] = json!(sk);

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let path = format!("/tmp/test-account-{:?}.json", timestamp.as_millis());
    // let path = "/tmp/map-test-account.json";
    let file = fs::File::create(path.clone()).unwrap();
    serde_json::to_writer(file, &account_json).unwrap();

    Account::from_file(path)
}

async fn deploy_and_init_wnear(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(WNEAR_WASM_FILEPATH)?).await?;
    println!("deploy wnear contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init WNEAR contract failed!");

    Ok(contract)
}

async fn deploy_mcs_token_and_set_decimals(worker: &Worker<Sandbox>, mcs: &Contract, token_name: String, decimals: u8) -> anyhow::Result<()> {
    let res = gen_call_transaction(worker, mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);

    let token_account = format!("{}.{}", token_name, mcs.id().to_string());

    let res = gen_call_transaction(worker, mcs, "set_metadata", json!({"address": token_account, "decimals": decimals}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_metadata for {} failed", token_name);

    Ok(())
}

async fn deploy_and_init_ft(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    deploy_and_init_ft_with_decimal(worker, 24).await
}

async fn deploy_and_init_ft_with_decimal(worker: &Worker<Sandbox>, decimal: u8) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MCS_TOKEN_WASM_FILEPATH)?).await?;
    println!("deploy ft contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({"owner": contract.id()}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init ft contract failed!");

    let res = gen_call_transaction(worker, &contract, "set_metadata", json!({"address": contract.id(), "decimals": decimal}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "set_metadata for {} failed", contract.id());

    Ok(contract)
}

async fn deploy_and_init_ft_with_account(worker: &Worker<Sandbox>, account: &Account) -> anyhow::Result<Contract> {
    let contract = account.deploy(&worker, &std::fs::read(MCS_TOKEN_WASM_FILEPATH)?).await?.unwrap();
    println!("deploy ft contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({"owner": contract.id()}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init ft contract failed!");

    Ok(contract)
}

async fn deploy_and_init_mcs(worker: &Worker<Sandbox>, map_light_client: String, map_bridge_address: String, wrapped_token: String) -> anyhow::Result<Contract> {
    let token_account: AccountId = "mcs.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let contract = account.deploy(&worker, &std::fs::read(MCS_WASM_FILEPATH)?).await?.unwrap();
    println!("deploy mcs contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "init")
        .args_json(json!({"owner": contract.id(),
            "map_light_client": map_light_client,
            "map_bridge_address":map_bridge_address,
            "wrapped_token": wrapped_token,
            "near_chain_id": "1313161555",
        "map_chain_id": "22776"}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    Ok(contract)
}

async fn deploy_and_init_light_client(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MOCK_MAP_CLIENT_WASM_FILEPATH)?).await?;
    println!("deploy map light client contract id: {:?}", contract.id());

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    Ok(contract)
}

async fn deploy_and_init_multisig(worker: &Worker<Sandbox>, init_args: serde_json::Value) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MULTISIG_WASM_FILEPATH)?).await?;
    println!("deploy multisig contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    Ok(contract)
}

async fn init_worker() -> anyhow::Result<Worker<Sandbox>> {
    std::env::var(NEAR_SANDBOX_BIN_PATH).expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::sandbox().await?;

    Ok(worker)
}