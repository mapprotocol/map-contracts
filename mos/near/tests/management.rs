use near_sdk::json_types::U128;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::types::{KeyType, SecretKey};
use workspaces::{prelude::*, AccountId};

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn test_owner() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;

    let account_id: AccountId = "mcs.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let owner = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let mcs = worker
        .dev_deploy(&std::fs::read(MCS_WASM_FILEPATH)?)
        .await?;
    println!("deploy mcs contract id: {:?}", mcs.id());

    let res = mcs
        .call(&worker, "init")
        .args_json(json!({"owner": owner.id(),
            "map_light_client": "map_light_client.near",
            "map_bridge_address":MAP_BRIDGE_ADDRESS.to_string(),
            "wrapped_token": wnear.id().to_string(),
            "near_chain_id": "1313161555",
            "map_chain_id": "22776",
            "ref_exchange": "ref.near",
            "butter_core": [],
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    let chain_id: U128 = U128(100);
    let token_account: AccountId = "mcs_token.test.near".parse().unwrap();
    let token = deploy_mcs_token_and_set_decimals(&worker, &mcs, &token_account, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "register_token",
        json!({"token": token.id().to_string(), "mintable": true}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_mcs_token_to_chain",
        json!({"token": token_account, "to_chain": chain_id}),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "add_mcs_token_to_chain should be called by owner"
    );

    let res = gen_call_transaction_by_account(
        &worker,
        &owner,
        &mcs,
        "add_mcs_token_to_chain",
        json!({"token": token_account, "to_chain": chain_id}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": chain_id, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await;
    assert!(res.is_err(), "set_chain_type should be called by owner");

    let res = gen_call_transaction_by_account(
        &worker,
        &owner,
        &mcs,
        "set_chain_type",
        json!({"chain_id": chain_id, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;
    let res = gen_call_transaction(
        &worker,
        &mcs,
        "register_token",
        json!({"token": ft.id().to_string(), "mintable": false}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let token_name0 = ft.id().to_string();
    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name0, "to_chain": chain_id}),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "add_fungible_token_to_chain should be called by owner"
    );

    let res = gen_call_transaction_by_account(
        &worker,
        &owner,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name0, "to_chain": chain_id}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed"
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": chain_id }),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "add_native_to_chain should be called by owner"
    );

    let res = gen_call_transaction_by_account(
        &worker,
        &owner,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": chain_id }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    Ok(())
}

#[tokio::test]
async fn test_manage_to_chain_type() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        "map_light_client.near".to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let chain_id: U128 = U128(100);
    let chain_type = ChainType::EvmChain;

    let ret = gen_call_transaction(
        &worker,
        &mcs,
        "get_chain_type",
        json!({ "chain_id": chain_id }),
        false,
    )
    .view()
    .await?
    .json::<ChainType>()?;
    assert_eq!(ChainType::Unknown, ret, "chain type should be unknonw");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": chain_id, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let ret = gen_call_transaction(
        &worker,
        &mcs,
        "get_chain_type",
        json!({ "chain_id": chain_id }),
        false,
    )
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
    let mcs = worker
        .dev_deploy(&std::fs::read(MCS_WASM_FILEPATH)?)
        .await?;
    println!("deploy mcs contract id: {:?}", mcs.id());

    let res = mcs
        .call(&worker, "init")
        .args_json(json!({"owner": mcs.id(),
            "map_light_client": "map_light_client.near".to_string(),
            "map_bridge_address":MAP_BRIDGE_ADDRESS.to_string(),
            "wrapped_token": wnear.id().to_string(),
            "near_chain_id": "5566818579631833089",
        "map_chain_id": "22776",
        "ref_exchange": "ref.near",
        "butter_core": []
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    let near_chain_id: U128 = U128(5566818579631833088);
    let paused_mask = 1 << 2 | 1 << 3 | 1 << 4 | 1 << 5;

    let ret = gen_call_transaction(&worker, &mcs, "get_near_chain_id", json!({}), false)
        .view()
        .await?
        .json::<U128>()?;

    println!("get_near_chain_id {}", ret.0);
    assert_eq!(5566818579631833089, ret.0, "get default near chain id");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_near_chain_id",
        json!({ "near_chain_id": near_chain_id }),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "set_chain_type should fail because of not paused"
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_paused",
        json!({ "paused": paused_mask }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_paused should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_near_chain_id",
        json!({ "near_chain_id": near_chain_id }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let ret = gen_call_transaction(&worker, &mcs, "get_near_chain_id", json!({}), false)
        .view()
        .await?
        .json::<U128>()?;
    println!("get near chain id {}", ret.0);
    assert_eq!(near_chain_id, ret, "near chain id should be set");

    Ok(())
}

#[tokio::test]
async fn test_manage_map_relay_address() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        "map_light_client.near".to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let map_relay_address = "aaaa5a86411ab8627516cbb77d5db00b74fe610d";
    let paused_mask = 1 << 1;

    let ret = gen_call_transaction(&worker, &mcs, "get_map_relay_address", json!({}), false)
        .view()
        .await?
        .json::<String>()?;
    assert_eq!(
        MAP_BRIDGE_ADDRESS.to_string(),
        ret,
        "get default map relay address"
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_map_relay_address",
        json!({ "map_relay_address": map_relay_address }),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "set_map_relay_address should fail because of not paused"
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_paused",
        json!({ "paused": paused_mask }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_paused should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_map_relay_address",
        json!({ "map_relay_address": map_relay_address }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_map_relay_address should succeed");

    let ret = gen_call_transaction(&worker, &mcs, "get_map_relay_address", json!({}), false)
        .view()
        .await?
        .json::<String>()?;
    println!("get map relay address {}", ret);
    assert_eq!(
        map_relay_address.to_string(),
        ret,
        "map relay address should be set"
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_map_relay_address",
        json!({"map_relay_address": "aaaa5a86411ab8627516cbb77d5db00b74f"}),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "set_map_relay_address should fail because of invalid eth address"
    );

    Ok(())
}
