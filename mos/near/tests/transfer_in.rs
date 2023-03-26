use near_sdk::json_types::U128;
use std::fs;
// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::types::{KeyType, SecretKey};
use workspaces::{prelude::*, AccountId};

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn test_transfer_in_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.testnet".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
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
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("unexpected map mcs address"),
        "should be mcs address error"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_no_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("is not mcs token or fungible token or native token"),
        "token is invalid"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_no_light_client() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        "map_light_client.near".to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "verify_receipt_proof should fail");
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("verify proof failed"),
        "should be cross contract call error"
    );
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("receipt proof has not been verified yet"),
        "should be proof not verified error"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT - 1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("not enough deposit for record proof"),
        "should be deposit error"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
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
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("not enough deposit for mcs token mint"),
        "should be deposit error"
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 > dev_balance_0 - dev_balance_1);
    assert!(dev_balance_1 - dev_balance_2 < dev_balance_0 - dev_balance_1 + ORDER_ID_DEPOSIT / 2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let to: AccountId = "pandarr.testnet".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_3
    );
    println!("dev_account decrease {}", dev_balance_2 - dev_balance_3); //35777116013107000000000
    assert!(
        dev_balance_2 - dev_balance_3
            > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1250000000000000000000
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    let balance_0 = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance_0.0, "balance of {} is incorrect", to);

    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(
        res.err().unwrap().to_string().contains("is used"),
        "transfer in should failed because of used proof"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let balance_1 = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(balance_0.0, balance_1.0, "balance of {} is incorrect", to);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_mcs_token_invalid_address() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let file = fs::File::open("./tests/data/transfer_in_token.json").unwrap();
    let mut proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    proof["receipt"]["logs"][0]["data"] = json!("0x64e604787cbf194841e7b68d7cd28786f6c9a0a3ab9f8b0a0e87cb4387ab010700000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000000000000000000000000000000000000000000000000000000000000000000014223e016916ba9f10762e33e03e8556409d096f220000000000000000000000000000000000000000000000000000000000000000000000000000000000000006010203040506000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000196d63735f746f6b656e5f302e6d63732e746573742e6e65617200000000000000");
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err().unwrap());
    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("invalid to address"));

    proof["receipt"]["logs"][0]["data"] = json!("0x64e604787cbf194841e7b68d7cd28786f6c9a0a3ab9f8b0a0e87cb4387ab010700000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000000000000000000000000000000000000000000000000000000000000000000014223e016916ba9f10762e33e03e8556409d096f22000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f70616e646172722e746573746e6574000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000070102030405060700000000000000000000000000000000000000000000000000");

    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err().unwrap());
    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("invalid to chain token address"));
    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let to_chain = U128(1);
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(1000);
    let res = ft
        .call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(900, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_no_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(1000);
    let res = ft
        .call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total, balance, "balance of mcs is incorrect");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "register_token",
        json!({"token": token_account, "mintable": false}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let to_chain = U128(1);
    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_account, "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed"
    );

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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - 100, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let to_chain = U128(1);
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let total: U128 = U128::from(1000);
    let res = ft
        .call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let dev_account = worker.dev_create_account().await?;
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT - 1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should failed");
    println!("{}", res.as_ref().err().unwrap().to_string());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("not enough deposit for record proof"),
        "should be deposit error"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
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
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("not enough deposit for ft transfer"),
        "should be deposit error"
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 > dev_balance_0 - dev_balance_1);
    assert!(dev_balance_1 - dev_balance_2 < dev_balance_0 - dev_balance_1 + ORDER_ID_DEPOSIT / 2);
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
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
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_3
    );
    println!("dev_account decrease {}", dev_balance_2 - dev_balance_3); //35777116013107000000000
    assert!(dev_balance_2 - dev_balance_3 > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT);
    assert!(
        dev_balance_2 - dev_balance_3
            < dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + parse_near!("1 N")
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0 - 100, balance.0, "balance of mcs is incorrect");

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_ft_token_not_enough_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let to_chain = U128(1);
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let file = fs::File::open("./tests/data/transfer_in_ft_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let total: U128 = U128::from(10);
    let res = ft
        .call(&worker, "mint")
        .args_json(json!({"account_id": mcs.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");

    let dev_account = worker.dev_create_account().await?;
    let dev_balance_0 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("transfer in token failed"),
        "should be transfer in token error"
    );
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let to: AccountId = "pandarr.test.near".parse().unwrap();
    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(total.0, balance.0, "balance of mcs is incorrect");

    let total1: U128 = U128::from(1000);
    let res = ft
        .call(&worker, "mint")
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
    println!(
        "after transfer in 2: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((to.clone(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(100, balance.0, "balance of {} is incorrect", to);

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total.0 + total1.0 - 100,
        balance.0,
        "balance of mcs is incorrect"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        account_id, balance_0
    );
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(
        amount,
        balance_1 - balance_0,
        "should transfer in 100 yocto near"
    );
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(amount, mcs_wnear_0.0 - mcs_wnear_1.0);

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        account_id, balance_0
    );
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
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
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(balance_0, balance_0, "to account balance should not change");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(
        mcs_wnear_0.0, mcs_wnear_1.0,
        "wnear balance of mcs should not change"
    );

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(ORDER_ID_DEPOSIT - 1)
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
        .deposit(ORDER_ID_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());

    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!(
        "after transfer in 3: account {} balance: {}",
        account_id, balance_2
    );
    assert_eq!(
        balance_1 + amount,
        balance_2,
        "should transfer in {} yocto near",
        amount
    );
    let dev_balance_3 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in 3: account {} balance: {}",
        dev_account.id(),
        dev_balance_3
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_1 - dev_balance_3
    );
    assert!(dev_balance_2 - dev_balance_3 > dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1);
    assert!(
        dev_balance_2 - dev_balance_3
            < dev_balance_1 - dev_balance_2 + ORDER_ID_DEPOSIT + 1 + parse_near!("1 N")
    );

    let mcs_wnear_2 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_2.0
    );
    assert_eq!(
        mcs_wnear_0.0 - amount,
        mcs_wnear_2.0,
        "wnear balance of mcs should decrease {}",
        amount
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_wnear() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(99)
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );
    assert!(mcs_wnear_0.0 < amount, "should less than expected amount");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        account_id, balance_0
    );

    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(balance_0, balance_1, "account should get 0 near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(
        mcs_wnear_0.0, mcs_wnear_1.0,
        "wnear for mcs should not change"
    );

    // deposit to wnear and transfer in again
    let res = mcs
        .as_account()
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
    println!(
        "after transfer in 2: account {} balance: {}",
        account_id, balance_2
    );
    assert_eq!(
        balance_1 + amount,
        balance_2,
        "account should get {} yocto near",
        amount
    );
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in 2: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_1 - dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in 2: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_2.0
    );
    assert_eq!(
        mcs_wnear_1.0 + 100 - amount,
        mcs_wnear_2.0,
        "wnear for mcs should decrease {} tocto near",
        amount
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_no_to_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );

    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("transfer in token failed, maybe TO account does not exist"));
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(
        mcs_wnear_0.0, mcs_wnear_1.0,
        "wnear of mcs should not change"
    );

    // create account and transfer in again should still failed
    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in 2: account {} balance: {}",
        account_id, balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err().unwrap());
    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("the event with order id"));
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!(
        "after transfer in 2: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(0, balance_1 - balance_0, "should transfer in 0 yocto near");
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in 2: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_1 - dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in 2: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_2.0
    );
    assert_eq!(
        mcs_wnear_1.0, mcs_wnear_2.0,
        "wnear of mcs should decrease 0 yocto near"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_replay() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        account_id, balance_0
    );
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
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
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(
        amount,
        balance_1 - balance_0,
        "should transfer in 100 yocto near"
    );
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
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
    println!(
        "after transfer in 2: account {} balance: {}",
        account_id, balance_2
    );
    assert_eq!(balance_1, balance_2, "to account balance should not change");
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in 2: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_1 - dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in 2: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_2.0
    );
    assert_eq!(
        mcs_wnear_1.0, mcs_wnear_2.0,
        "wnear balance of mcs should not change"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_in_native_token_not_enough_gas() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let file = fs::File::open("./tests/data/transfer_in_native.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();
    let amount: u128 = 100;

    let dev_account = worker.dev_create_account().await?;
    let res = mcs
        .as_account()
        .call(&worker, wnear.id(), "near_deposit")
        .gas(300_000_000_000_000)
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "near withdraw should success");

    let mcs_wnear_0 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_0.0
    );

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let balance_0 = worker.view_account(&account_id).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        account_id, balance_0
    );
    let dev_balance_0 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "before transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_0
    );
    let res = dev_account
        .call(&worker, mcs.id(), "verify_receipt_proof")
        .args_json(json!({ "receipt_proof": proof }))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "verify_receipt_proof should succeed");
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(40_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await;
    assert!(res.is_err(), "transfer_in should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("Exceeded the prepaid gas"),
        "should be gas error"
    );
    let balance_1 = worker.view_account(&account_id).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_1
    );
    assert_eq!(balance_1, balance_0, "should transfer in 0 yocto near");
    let dev_balance_1 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_0 - dev_balance_1
    );
    assert!(dev_balance_0 - dev_balance_1 < parse_near!("1 N"));

    let mcs_wnear_1 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(mcs_wnear_0.0, mcs_wnear_1.0);

    let res = dev_account
        .call(&worker, mcs.id(), "transfer_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(200_000_000_000_000)
        .deposit(parse_near!("4 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_in should succeed");
    println!("logs {:?}", res.logs());
    let balance_2 = worker.view_account(&account_id).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        account_id, balance_2
    );
    assert_eq!(
        amount,
        balance_2 - balance_1,
        "should transfer in {} yocto near",
        amount
    );
    let dev_balance_2 = worker.view_account(&dev_account.id()).await?.balance;
    println!(
        "after transfer in: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!(
        "dev account balance decrease: {}",
        dev_balance_1 - dev_balance_2
    );
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));

    let mcs_wnear_2 = dev_account
        .call(&worker, wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer in: account {} wnear balance: {}",
        mcs.id(),
        mcs_wnear_1.0
    );
    assert_eq!(amount, mcs_wnear_1.0 - mcs_wnear_2.0);
    Ok(())
}
