use hex;
use near_sdk::json_types::U128;
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
async fn test_transfer_out_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(5566818579631833089);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res.logs());
    assert!(
        res.logs().get(1).unwrap().contains(TRANSFER_OUT_TYPE),
        "can not get transfer out log"
    );
    assert!(
        res.logs().get(0).unwrap().contains("5566818579631833089"),
        "can not correct to chain id"
    );
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total.0 - amount.0,
        balance.0,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());
    assert_ne!(
        res.logs().get(1).unwrap(),
        res1.logs().get(1).unwrap(),
        "log should be different"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_no_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(100);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err().unwrap());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"),
        "should be 1 yoctoNEAR error"
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total.0,
        balance.0,
        "balance of {} is incorrect",
        dev_account.id()
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_invalid_to_account() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F945").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("address length is incorrect for evm chain type"),
        "should be address error"
    );
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("amount too small"),
        "should be amount too small error"
    );
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let amount: U128 = U128(1000000000000000000000);
    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 18).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    let amount: U128 = U128(1000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to.clone(), amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("amount too small"),
        "should be amount too small error"
    );
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let amount: U128 = U128(1000000000000000);
    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json((token_account.to_string(), to, amount, to_chain))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_too_much_deposit() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
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
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    assert!(mcs_balance_2 - mcs_balance_1 < parse_near!("1 N"));
    assert!(dev_balance_1 - dev_balance_2 < parse_near!("1 N"));
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total.0,
        balance.0,
        "balance of {} is incorrect",
        dev_account.id()
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1001);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let to_chain: U128 = U128(1000);
    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let dev_account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let total: U128 = U128::from(1000000000000000000000000000);
    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_0
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs_balance_1
    );
    assert!(mcs_balance_0 - mcs_balance_1 < parse_near!("1 N"));

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let dev_balance_1 = dev_account.view_account(&worker).await?.balance;
    println!(
        "before transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_1
    );
    let amount: U128 = U128(100000000000000000000000000);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("is not supported"),
        "should be to chain not support error"
    );
    let mcs_balance_2 = mcs.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        mcs.id(),
        mcs_balance_2
    );
    let dev_balance_2 = dev_account.view_account(&worker).await?.balance;
    println!(
        "after transfer out: account {} balance: {}",
        dev_account.id(),
        dev_balance_2
    );
    println!("mcs balance increase: {}", mcs_balance_2 - mcs_balance_1);
    assert!(mcs_balance_2 > mcs_balance_1);
    assert!(dev_balance_1 > dev_balance_2);
    println!(
        "{}, {}",
        (dev_balance_1 - dev_balance_2) * 3 / 10,
        mcs_balance_2 - mcs_balance_1
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total.0,
        balance.0 as u128,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_mcs_token_to_chain",
        json!({"token": token_account.to_string(), "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let res1 = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res1.is_success(), "transfer_out_token should succeed");
    println!("logs {:?}", res1.logs());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_mcs_token_burn_failed() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let dev_account = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    println!(
        "before mint: account {} balance: {}",
        mcs.id(),
        mcs.view_account(&worker).await?.balance
    );
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": dev_account.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(30000000000000000000000)
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!(
        "after mint: account {} balance: {}",
        mcs.id(),
        mcs.view_account(&worker).await?.balance
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: U128 = U128(1000000000000000000000000000 + 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_token should fail");
    println!("error: {}", res.as_ref().err().unwrap().to_string());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("burn mcs token or call near_deposit()"),
        "should be burn mcs token or call near_deposit() error"
    );

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        total,
        balance,
        "balance of {} is incorrect",
        dev_account.id()
    );

    let amount: U128 = U128(1000000000000000000000000000 - 1);
    let res = dev_account
        .call(&worker, mcs.id(), "transfer_out_token")
        .args_json(json!({"token": token_account.to_string(), "to": to, "amount": amount, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "transfer_out_token should succeed");
    println!("logs: {:?}", res.logs());

    let balance = dev_account
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((dev_account.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(1, balance.0, "balance of {} is incorrect", dev_account.id());

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(3, res.logs().len(), "should have 3 logs");

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of root account is {:?}",
        balance
    );
    assert_eq!(
        total.0 - amount.0,
        balance.0 as u128,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = "abc".as_bytes().to_vec();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total.0 - amount.0,
        balance.0,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(1000);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, Some(18)).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(1000000000000000 - 1);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total.0 - amount.0,
        balance.0,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(amount, balance, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_fungible_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(100);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, Some(18)).await?;

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let to_chain: U128 = U128(1000);
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("ft_transfer_call logs: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = prepare_mcs(&worker).await?;
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;

    let to_chain: U128 = U128(1000);
    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount": total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        balance
    );
    assert_eq!(
        total, balance,
        "before transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("before transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "before transfer out ft balance of mcs");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Transfer { to, to_chain };
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, ft.id(), "ft_transfer_call")
        .args_json((
            mcs.id().to_string(),
            amount,
            Option::<String>::None,
            serde_json::to_string(&msg).unwrap(),
        ))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "ft_transfer_call should fail");
    println!("ft_transfer_call error: {:?}", res.err());

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after transfer out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after transfer out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await?;

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": to_chain }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
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
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        amount < balance_from_0 - balance_from_1,
        "sender's balance should decrease more than {}",
        amount
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        amount, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_invalid_account() -> anyhow::Result<()> {
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

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("0.001 N");
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31").unwrap();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": to_chain }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
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
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        balance_from_0 - balance_from_1 < parse_near!("1 N"),
        "sender's balance decrease too much"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_too_small() -> anyhow::Result<()> {
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

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("0.0009 N");
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": to_chain }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
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
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        balance_from_0 - balance_from_1 < parse_near!("1 N"),
        "sender's balance decrease too much"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

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
    println!(
        "after transfer out native 2, account {} balance {}",
        from.id(),
        balance_from_2
    );
    assert!(
        balance_from_1 - balance_from_2 > amount,
        "sender's balance should decrease more than {}",
        amount
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        amount, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_no_deposit() -> anyhow::Result<()> {
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

    let from = worker.dev_create_account().await?;
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": to_chain }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(300_000_000_000_000)
        .deposit(0)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("amount should > 0"),
        "should be deposit error"
    );

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        balance_from_0 > balance_from_1,
        "sender's balance should decrease more than 0"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

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
    println!(
        "after transfer out native 2, account {} balance {}",
        from.id(),
        balance_from_2
    );
    assert!(
        balance_from_0 - balance_from_1 > amount,
        "sender's balance should decrease more than 1"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        amount, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );
    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_no_to_chain() -> anyhow::Result<()> {
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

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
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
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        balance_from_0 - balance_from_1 < parse_near!("1 N"),
        "sender's balance decrease too much"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

    Ok(())
}

#[tokio::test]
async fn test_transfer_out_native_not_enough_gas() -> anyhow::Result<()> {
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

    let from = worker.dev_create_account().await?;
    let amount: u128 = parse_near!("10 N");
    let to_chain: U128 = U128(1000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_native_to_chain",
        json!({ "to_chain": to_chain }),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_native_to_chain should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "set_chain_type",
        json!({"chain_id": to_chain, "chain_type": "EvmChain"}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "set_chain_type should succeed");

    let balance_from_0 = from.view_account(&worker).await?.balance;
    println!(
        "before transfer out native, account {} balance {}",
        from.id(),
        balance_from_0
    );
    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(15_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_1 = from.view_account(&worker).await?.balance;
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_1
    );
    assert!(
        balance_from_0 - balance_from_1 < parse_near!("1 N"),
        "sender's balance decrease too much"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

    let res = from
        .call(&worker, mcs.id(), "transfer_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain}))?
        .gas(22_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "transfer_out_native should fail");
    println!("error: {:?}", res.err());

    let balance_from_2 = from.view_account(&worker).await?.balance;
    println!(
        "after transfer out native, account {} balance {}",
        from.id(),
        balance_from_2
    );
    assert!(
        balance_from_1 - balance_from_2 < parse_near!("1 N"),
        "sender's balance decrease too much"
    );

    let balance = wnear
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        0, balance.0,
        "wnear balance of mcs contract account == transferred out native token amount"
    );

    Ok(())
}
