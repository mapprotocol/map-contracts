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
use workspaces::{prelude::*, Worker, Contract, AccountId};
use workspaces::network::Sandbox;
use workspaces::operations::CallTransaction;
use workspaces::result::CallExecutionDetails;
use map_light_client::{EpochRecord, Validator};
use mcs::FungibleTokenMsg;

const MAP_CLIENT_WASM_FILEPATH: &str = "../../mapclients/near/target/wasm32-unknown-unknown/release/map_light_client.wasm";
const MCS_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs.wasm";
const MCS_TOKEN_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mcs_token.wasm";
const WNEAR_WASM_FILEPATH: &str = "./tests/data/w_near.wasm";
const NEAR_SANDBOX_BIN_PATH: &str = "NEAR_SANDBOX_BIN_PATH";


#[tokio::test]
async fn test_deploy_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                       "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                       "wrap.near".to_string()).await?;
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        let res = contract
            .call(&worker, "deploy_mcs_token")
            .args_json(json!({"address": token_name}))?
            .gas(300_000_000_000_000)
            .deposit(7_000_000_000_000_000_000_000_000)
            .transact()
            .await?;
        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);
    }

    let tokens = contract
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
    let contract = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                       "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                       "wrap.near".to_string()).await?;
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);

        let res = gen_call_transaction(&worker, &contract, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await;
        assert!(res.is_err(), "add_mcs_token_to_chain should fail since it is not deployed");

        let res = gen_call_transaction(&worker, &contract, "deploy_mcs_token", json!({"address": token_name}), true)
            .transact()
            .await?;
        assert!(res.is_success(), "deploy_mcs_token {} failed", token_name);

        let res = gen_call_transaction(&worker, &contract, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "add_mcs_token_to_chain should succeed since it has been deployed");

        let is_valid = gen_call_transaction(&worker, &contract, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(is_valid, "mcs token {} to chain {} should be valid", token_name, i);

        let is_valid = gen_call_transaction(&worker, &contract, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i+1}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "mcs token {} to chain {} should be invalid", token_name, i+1);

        let res = gen_call_transaction(&worker, &contract, "remove_mcs_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "remove_mcs_token_to_chain should succeed");

        let is_valid = gen_call_transaction(&worker, &contract, "valid_mcs_token_out", json!({"token": token_name, "to_chain": i}), false)
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
    let contract = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                       "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                       "wrap.near".to_string()).await?;
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);

        let is_valid = gen_call_transaction(&worker, &contract, "valid_fungible_token_out", json!({"token": token_name, "to_chain": i+1}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name, i);

        let res = gen_call_transaction(&worker, &contract, "add_fungible_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "add_fungible_token_to_chain should succeed since it has been deployed");

        let is_valid = gen_call_transaction(&worker, &contract, "valid_fungible_token_out", json!({"token": token_name, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(is_valid, "fungible token {} to chain {} should be valid", token_name, i);

        let is_valid = gen_call_transaction(&worker, &contract, "valid_fungible_token_out", json!({"token": token_name, "to_chain": i+1}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name, i+1);
    }

    let tokens = contract
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<String>>()?;

    assert_eq!(2, tokens.len(), "wrong fungible tokens size");
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        assert!(tokens.contains(&token_name), "{} is not contained", token_name)
    }

    for i in 0..2 {
        let token_name = format!("eth_token{}", i);

        let res = gen_call_transaction(&worker, &contract, "remove_fungible_token_to_chain", json!({"token": token_name, "to_chain": i}), false)
            .transact()
            .await?;
        assert!(res.is_success(), "remove_fungible_token_to_chain should succeed");

        let is_valid = gen_call_transaction(&worker, &contract, "valid_fungible_token_out", json!({"token": token_name, "to_chain": i}), false)
            .view()
            .await?
            .json::<bool>()?;
        assert!(!is_valid, "fungible token {} to chain {} should be invalid", token_name, i);
    }

    let tokens = contract
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<String>>()?;

    assert_eq!(2, tokens.len(), "wrong fungible tokens size");
    for i in 0..2 {
        let token_name = format!("eth_token{}", i);
        assert!(tokens.contains(&token_name), "{} is not contained", token_name)
    }


    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract1
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract1
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "transfer_in failed");

    let to: AccountId = "6feacd7ddb1bf2511b4ea0e83d89be0af295d52adb965883283d6835b15e0cd3".parse().unwrap();
    let token_account = AccountId::from_str(format!("{}.{}", token_name, contract1.id().to_string()).as_str()).unwrap();
    let contract2 = worker.import_contract(&token_account, &worker).transact().await?;

    let balance = contract2
        .call(&worker, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;

    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_no_light_client() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract = deploy_and_init_mcs(&worker, "map_light_client.near".to_string(),
                                       "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                       "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    assert!(res.err().unwrap().to_string().contains("get result from cross contract"), "should be cross contract call error");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_deposit_0() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract1
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract1
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    assert!(res.err().unwrap().to_string().contains("not enough deposit for record proof"), "should be cross contract call error");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract1
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract1
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "transfer_in failed");

    let res = contract1
        .call(&worker, "transfer_in")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    assert!(res.err().unwrap().to_string().contains("is used"), "transfer in should failed because of used proof");

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract1
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let to_chain: u64 = 1000;
    let res = gen_call_transaction(&worker, &contract1, "add_mcs_token_to_chain", json!({"token": token_name, "to_chain": to_chain}), false)
        .transact()
        .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let from = worker.root_account().id().to_string();
    let amount: u64 = 40;
    let res = contract1
        .call(&worker, "mint")
        .args_json(json!({"token": token_name, "to": from, "amount": amount}))?
        .gas(300_000_000_000_000)
        .deposit(70000000000000000000000)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");

    let token_account = AccountId::from_str(format!("{}.{}", token_name, contract1.id().to_string()).as_str()).unwrap();
    let contract2 = worker.import_contract(&token_account, &worker).transact().await?;
    let balance = contract2
        .call(&worker, "ft_balance_of")
        .args_json((from.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(amount, balance.0 as u64, "balance of {} is incorrect", from);

    let to_chain: u64 = 1000;
    let to = "abcd".to_string();
    log!("before transfer out: {}", contract1.view_account(&worker).await?.balance);
    let res = worker.root_account()
        .call(&worker, contract1.id(),"transfer_out_token")
        .args_json(json!({"token": token_name, "to": to, "amount": 10, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.logs().get(0).unwrap().contains("finish_transfer_out"), "get expected log");
    assert!(res.is_success(), "transfer_out_token failed");
    log!("after transfer out: {}", contract1.view_account(&worker).await?.balance);

    let contract2 = worker.import_contract(&token_account, &worker).transact().await?;
    let balance = contract2
        .call(&worker, "ft_balance_of")
        .args_json((from.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(amount - 10, balance.0 as u64, "balance of {} is incorrect", from);

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    let ft = deploy_and_init_ft(&worker).await?;
    let mcs = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                  "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                  "wrap.near".to_string()).await?;

    let amount: u64 = 10000;
    let amount_str = amount.to_string();
    let to_chain: u64 = 1000;
    let to = "abcd".to_string();
    let from = worker.root_account();
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let res = ft
        .call(&worker, "mint")
        .args_json(json!({"account_id": from.id(), "amount": amount_str}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;

    let fungible_token = ft.id().to_string();
    let res = mcs
        .call(&worker, "add_fungible_token_to_chain")
        .args_json(json!({"token": fungible_token, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "add_fungible_token_to_chain failed");

    let msg = FungibleTokenMsg {
        typ: 0,
        to: to.clone(),
        to_chain: to_chain as _,
    };

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((worker.root_account().id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of root account is {:?}", balance);
    assert_eq!(amount, balance.0 as u64, "before transfer out ft balance of root account");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let res = worker.root_account()
        .call(&worker, ft.id(), "storage_deposit")
        .args_json(json!({"account_id":mcs.id().to_string(), "registration_only": true}))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "storage_deposit failed");

    let res = worker.root_account()
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((mcs.id().to_string(), "10", Option::<String>::None, serde_json::to_string(&msg).unwrap()))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(2, res.logs().len(), "should have 2 logs");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((worker.root_account().id().to_string(), ))?
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
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract2 = deploy_and_init_wnear(&worker).await?;
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        contract2.id().to_string()).await?;

    let from = worker.root_account().id().to_string();
    let amount: u128 = 70000000000000000000000;
    let to_chain: u64 = 1000;
    let to = "abcd".to_string();

    let from = worker.root_account();
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, contract1.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_native failed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;
    // FIXME
    // assert_eq!(amount, balance_from_0 - balance_from_1, "sender's balance should decrease {}", amount);

    let balance = contract2
        .call(&worker, "ft_balance_of")
        .args_json((contract1.id().to_string(), ))?
        .view()
        .await?
        .json::<U128>()?;
    assert!(amount - balance.0 > 0, "wnear balance of mcs contract account should be less than transferred out native token amount");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let amount: u128 = 70000000000000000000000;
    // let amount: u128 = 0;
    let to = "abcd".to_string();
    let from = worker.root_account();
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = contract1.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, contract1.id(), "deposit_out_native")
        .args_json(json!({"to": to}))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native failed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = contract1.view_account(&worker).await?.balance;
    log!("balance_from_0:{}, balance_from_1:{}", balance_from_0, balance_from_1);
    log!("balance_mcs_0:{}, balance_mcs_1:{}", balance_mcs_0, balance_mcs_1);

    log!("{}, {}", balance_from_0- balance_from_1, balance_mcs_1 - balance_mcs_0);
    assert!((balance_from_0- balance_from_1) > (balance_mcs_1 - balance_mcs_0));
    // FIXME
    // assert_eq!(amount, balance_from_0 - balance_from_1, "sender's balance should decrease {}", amount);

    Ok(())
}

// FIXME
#[tokio::test]
async fn test_mint() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let contract0 = deploy_and_init_light_client(&worker).await?;
    log!("{:?}", contract0.as_account().id().to_string());
    let contract1 = deploy_and_init_mcs(&worker, contract0.id().to_string(),
                                        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
                                        "wrap.near".to_string()).await?;

    let token_name = "eth_token";
    let res = contract1
        .call(&worker, "deploy_mcs_token")
        .args_json(json!({"address": token_name}))?
        .gas(300_000_000_000_000)
        .deposit(7_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "deploy_mcs_token failed");

    let to = "abcd".to_string();
    let amount: u64 = 50;
    log!("before transfer out: {}", contract1.view_account(&worker).await?.balance);
    // let res = contract1
    //     .call(&worker, "mint")
    //     .args_json(json!({"token": token_name, "to": to, "amount": amount}))?
    //     .gas(300_000_000_000_000)
    //     .deposit(70000000000000000000000)
    //     .transact()
    //     .await?;

    let res = worker.root_account().transfer_near(&worker, contract1.id(), 70000000000000000000000).await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "transfer_out_token failed");
    log!("after transfer out: {}", contract1.view_account(&worker).await?.balance);

    let token_account = AccountId::from_str(format!("{}.{}", token_name, contract1.id().to_string()).as_str()).unwrap();
    let contract2 = worker.import_contract(&token_account, &worker).transact().await?;
    let balance = contract2
        .call(&worker, "ft_balance_of")
        .args_json((to.clone(), ))?
        .view()
        .await?
        .json::<U128>()?;

    assert_eq!(50, balance.0, "balance of {} is incorrect", to);

    Ok(())
}

fn gen_call_transaction<'a, U: serde::Serialize>(worker: &'a Worker<Sandbox>, contract: &'a Contract, function: &'a str, args: U, deposit: bool) -> CallTransaction<'a, 'a, Sandbox> {
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

async fn deploy_and_init_ft(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
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


async fn deploy_and_init_mcs(worker: &Worker<Sandbox>, map_light_client: String, map_bridge_address: String, wrapped_token: String) -> anyhow::Result<Contract> {
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

async fn deploy_and_init_light_client(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let contract = worker.dev_deploy(&std::fs::read(MAP_CLIENT_WASM_FILEPATH)?).await?;
    println!("deploy light client contract id: {:?}", contract.id());

    let validators = r#"[
    {"g1_pub_key":{"x":"0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f249","y":"0x2b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3"},"weight":1,"address":"0xb4e1bc0856f70a55764fd6b3f8dd27f2162108e9"},
    {"g1_pub_key":{"x":"0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf58","y":"0x1ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b"},"weight":1,"address":"0x7a3a26123dbd9cfefc1725fe7779580b987251cb"},
    {"g1_pub_key":{"x":"0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c812","y":"0x1e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869"},"weight":1,"address":"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4"},
    {"g1_pub_key":{"x":"0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea6","y":"0x0dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"},"weight":1,"address":"0x65b3fee569bf82ff148bdded9c3793fb685f9333"},
    {"g1_pub_key":{"x":"0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d","y":"0x2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"},"weight":1,"address":"0x4ca1a81e4c46b90ec52371c063d5721df61e7e12"}
]"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(450);
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    println!("validators:{}", init_args["validators"]);
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
    std::env::set_var(NEAR_SANDBOX_BIN_PATH, "/Users/rong/Projects/near/nearcore/target/debug/neard-sandbox");
    std::env::var(NEAR_SANDBOX_BIN_PATH).expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::sandbox().await?;

    Ok(worker)
}