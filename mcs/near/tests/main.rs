use std::fmt::format;
use std::fs;
use std::ops::Index;
use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::{log, serde};
// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::{prelude::*, Worker, Contract, AccountId, Account, Network, DevNetwork};
use workspaces::network::{Sandbox, Testnet};
use workspaces::operations::CallTransaction;
use workspaces::result::CallExecutionDetails;
use map_light_client::{EpochRecord, Validator};
use mcs::FungibleTokenMsg;

const MOCK_MAP_CLIENT_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mock_map_client.wasm";
const MCS_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs.wasm";
const MCS_TOKEN_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs_token.wasm";
const WNEAR_WASM_FILEPATH: &str = "./tests/data/w_near.wasm";
const NEAR_SANDBOX_BIN_PATH: &str = "NEAR_SANDBOX_BIN_PATH";
const MAP_BRIDGE_ADDRESS: &str = "765a5a86411ab8627516cbb77d5db00b74fe610d";
const MAP_CHAIN_ID: u128 = 22776;


#[tokio::test]
async fn test_deploy_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
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
        .json::<Vec<String>>()?;

    assert_eq!(2, tokens.len(), "wrong mcs tokens size");
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        assert!(tokens.contains(&token_name), "{} is not contained", token_name)
    }

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

        let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await;
        assert!(res.is_err(), "add_mcs_token_to_chain should fail since it is not deployed");

        let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
            .transact()
            .await?;
        assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);

        let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "add_mcs_token_to_chain should succeed since it has been deployed");

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(is_valid, "mcs token {} to chain {} should be valid", token_name, i);

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i+1}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "mcs token {} to chain {} should be invalid", token_name, i + 1);

        let res = gen_call_transaction(&worker, &mcs, "remove_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "remove_mcs_token_to_chain should succeed");

        let is_valid = gen_call_transaction(&worker, &mcs, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "mcs token {} to chain {} should be invalid", token_name, i);
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
        .json::<Vec<String>>()?;
    assert_eq!(2, tokens.len(), "wrong fungible tokens size");
    assert!(tokens.contains(&token_name0), "{} is not contained", token_name0);
    assert!(tokens.contains(&token_name1), "{} is not contained", token_name1);

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
        .json::<Vec<String>>()?;
    assert_eq!(1, tokens.len(), "wrong fungible tokens size");
    assert!(tokens.contains(&token_name0), "{} is not contained", token_name0);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    assert!(res.is_success(), "transfer_in failed");

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
async fn test_transfer_in_token_wrong_bridge() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    assert!(res.is_err(), "transfer_in should fail");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("no cross chain event in the receipt"), "should have no event");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_no_token() -> anyhow::Result<()> {
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
    println!("before transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await;
    println!("after transfer in: account {} balance: {}", dev_account.id(), dev_account.view_account(&worker).await?.balance);
    assert!(res.is_err(), "transfer_in should fail");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("is not mcs token or fungible token or empty"), "token is invalid");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_no_light_client() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = mcs
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("get failed result from cross contract"), "should be cross contract call error");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_deposit_0() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = mcs
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("not enough deposit for record proof"), "should be cross contract call error");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker,
                                  map_client.id().to_string(),
                                  MAP_BRIDGE_ADDRESS.to_string(),
                                  wnear.id().to_string()).await?;

    let token_name = "mcs_token_0";
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = mcs
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "transfer_in failed");

    let res = mcs
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proof}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("is used"), "transfer in should failed because of used proof");

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
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let dev_account = worker.dev_create_account().await?;
    let total :U128 = U128::from(1000);
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
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

    let to = "abc".as_bytes().to_vec();
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_name, "to": to, "amount": 10, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.logs().get(0).unwrap().contains("transfer out"), "get expected log");
    assert!(res.is_success(), "transfer_out_token failed");

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - 10, balance.0 as u128, "balance of {} is incorrect", dev_account.id());

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
    let res = gen_call_transaction(&worker, &mcs, "deploy_mcs_token", json!({"address": token_name}), true)
        .transact()
        .await?;
    assert!(res.is_success(), "deploy_mcs_token failed");

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &mcs, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let dev_account = worker.dev_create_account().await?;
    let total :U128 = U128::from(1000);
    let token_account = AccountId::from_str(format!("{}.{}", token_name, mcs.id().to_string()).as_str()).unwrap();
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

    let to = "abc".as_bytes().to_vec();
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_name, "to": to, "amount": 1001, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(res.err().unwrap().to_string().contains("get failed result from cross contract"), "should be cross contract call error");

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

    let from = worker.dev_create_account().await?;
    let amount: u64 = 10000;
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": amount.to_string()}), true)
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
    assert_eq!(amount, balance.0 as u64, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = "abcd".as_bytes().to_vec();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        typ: 0,
        to,
        to_chain: to_chain as _,
    };
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), "10", Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
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
    assert_eq!(amount - 10, balance.0 as u64, "after transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(10, balance.0, "after transfer out ft balance of mcs");

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
    let to = "abcd".as_bytes().to_vec();

    let res = gen_call_transaction(&worker, &mcs, "add_native_to_chain", json!({"to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

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
    assert_eq!(amount,  balance.0, "wnear balance of mcs contract account == transferred out native token amount");

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
    let to = "abcd".as_bytes().to_vec();
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
    assert!(res.is_success(), "deposit_out_native failed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);
    log!("{}, {}", balance_from_0- balance_from_1, balance_mcs_1 - balance_mcs_0);
    assert!(balance_from_0- balance_from_1 > amount);
    assert!(balance_mcs_1 - balance_mcs_0 > amount);
    assert!((balance_from_0 - balance_from_1- amount) > (balance_mcs_1 - balance_mcs_0- amount));
    log!("{}", balance_from_0 - balance_from_1 - amount);
    log!("{}", balance_mcs_1 - balance_mcs_0 - amount);

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

    let to_chain : u64 = MAP_CHAIN_ID as _;
    let res = gen_call_transaction(&worker, &mcs, "add_fungible_token_to_chain", json!({"token": ft.id().to_string(), "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain should succeed");

    let from = worker.dev_create_account().await?;
    let amount: u64 = 10000;
    let res = gen_call_transaction(&worker, &ft, "mint", json!({"account_id": from.id(), "amount": amount.to_string()}), true)
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
    assert_eq!(amount, balance.0 as u64, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = "abcd".as_bytes().to_vec();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = FungibleTokenMsg {
        typ: 1,
        to,
        to_chain: 0 as _,
    };
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), "10", Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
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
    println!("after deposit out ft balance of root account is {:?}", balance);
    assert_eq!(amount - 10, balance.0 as u64, "after deposit out ft balance of from account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(10, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}


fn gen_call_transaction<'a, U: serde::Serialize>(worker: &'a Worker<impl DevNetwork>, contract: &'a Contract, function: &'a str, args: U, deposit: bool) -> CallTransaction<'a, 'a, impl DevNetwork> {
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

async fn deploy_and_init_wnear(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
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

async fn deploy_and_init_ft(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MCS_TOKEN_WASM_FILEPATH)?).await?;
    println!("deploy ft contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init ft contract failed!");

    Ok(contract)
}


async fn deploy_and_init_mcs(worker: &Worker<impl DevNetwork>, map_light_client: String, map_bridge_address: String, wrapped_token: String) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MCS_WASM_FILEPATH)?).await?;
    println!("deploy mcs contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "init")
        .args_json(json!({"map_light_client": map_light_client,
            "map_bridge_address":map_bridge_address,
            "wrapped_token": wrapped_token,
            "near_chain_id": 1313161555}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    Ok(contract)
}

async fn deploy_and_init_mcs1(worker: &Worker<impl DevNetwork>, account: &Account, map_light_client: String, map_bridge_address: String, wrapped_token: String) -> anyhow::Result<Contract> {
    let contract = account.deploy(worker, &std::fs::read(MCS_WASM_FILEPATH)?).await?.unwrap();
    // let contract = worker.dev_deploy(&std::fs::read(MCS_WASM_FILEPATH)?).await?;
    println!("deploy mcs contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "init")
        .args_json(json!({"map_light_client": map_light_client,
            "map_bridge_address":map_bridge_address,
            "wrapped_token": wrapped_token,
            "near_chain_id": 1313161555}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    Ok(contract)
}

async fn deploy_and_init_light_client(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
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

async fn deploy_and_init_light_client1(worker: &Worker<impl DevNetwork>, account: &Account) -> anyhow::Result<Contract> {
    let contract = account.deploy(worker, &std::fs::read(MOCK_MAP_CLIENT_WASM_FILEPATH)?).await?.unwrap();
    // let contract = worker.dev_deploy(&std::fs::read(MOCK_MAP_CLIENT_WASM_FILEPATH)?).await?;
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

async fn init_worker() -> anyhow::Result<Worker<impl DevNetwork>> {
    std::env::var(NEAR_SANDBOX_BIN_PATH).expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::sandbox().await?;

    Ok(worker)
}