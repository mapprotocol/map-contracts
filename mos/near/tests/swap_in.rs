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
async fn test_swap_in_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let owner = worker.dev_create_account().await?;
    let ref_exchange: AccountId = "ref.test.near".parse().unwrap();
    let cores: Vec<AccountId> = vec!["core.test.near".parse().unwrap()];
    let ref_exchange = prepare_ref_exchange(&worker, ref_exchange, &owner).await?;
    let mcs = prepare_mcs_with_ref_and_core(&worker, &wnear, &ref_exchange, cores).await?;

    let usdc_account: AccountId = "usdc.test.near".parse().unwrap();
    let to_chain: U128 = U128(100);
    let usdc = prepare_mos_token(&worker, &mcs, &usdc_account, to_chain, 24).await?;
    let amount: U128 = U128(10000000000000000000000000);

    let res = gen_call_transaction_by_account(
        &worker,
        &mcs.as_account(),
        &usdc,
        "mint",
        json!({"account_id": owner.id(), "amount": amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");

    near_deposit(&worker, &wnear, &owner, parse_near!("10 N")).await?;

    let _pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let _swap_amount: U128 = U128(3000000000000000000000000);

    let file = fs::File::open("./tests/data/swap_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap in: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = gen_call_transaction_by_account(
        &worker,
        &mcs.as_account(),
        &usdc,
        "mint",
        json!({"account_id": mcs.id(), "amount": amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");
    println!("mint usdc for mcs");

    let res = mcs
        .call(&worker, "swap_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "swap_in should succeed");
    println!("logs {:?}", res.logs());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after swap in: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    println!("user balance increase {}", user_balance_1 - user_balance_0);
    assert!(user_balance_1 - user_balance_0 > 0, "user balance increase");

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let mcs_balance_usdc_1 = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        amount.0 - mcs_balance_usdc_1.0,
        0,
        "mcs balance of usdc doesn't change"
    );

    Ok(())
}

#[tokio::test]
async fn test_swap_in_ft_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    let owner = worker.dev_create_account().await?;
    let ref_exchange: AccountId = "ref.test.near".parse().unwrap();
    let cores: Vec<AccountId> = vec!["core.test.near".parse().unwrap()];
    let ref_exchange = prepare_ref_exchange(&worker, ref_exchange, &owner).await?;
    let mcs = prepare_mcs_with_ref_and_core(&worker, &wnear, &ref_exchange, cores).await?;

    let usdc_account: AccountId = "usdc.test.near".parse().unwrap();
    let to_chain: U128 = U128(100);
    let usdc = prepare_ft(&worker, &mcs, &usdc_account, to_chain, None).await?;
    let amount: U128 = U128(10000000000000000000000000);

    let res = gen_call_transaction(
        &worker,
        &usdc,
        "mint",
        json!({"account_id": owner.id(), "amount": amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");

    near_deposit(&worker, &wnear, &owner, parse_near!("10 N")).await?;

    let _pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);

    let file = fs::File::open("./tests/data/swap_in_token.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap in: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = gen_call_transaction(
        &worker,
        &usdc,
        "mint",
        json!({"account_id": mcs.id(), "amount": amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");
    println!("mint usdc for mcs");

    let res = mcs
        .call(&worker, "swap_in")
        .args_json(json!({"receipt_proof": proof, "index": 0}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "swap_in should succeed");
    println!("logs {:?}", res.logs());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after swap in: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    println!("user balance increase {}", user_balance_1 - user_balance_0);
    assert!(user_balance_1 - user_balance_0 > 0, "user balance increase");

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let mcs_balance_usdc_1 = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        amount.0 - mcs_balance_usdc_1.0,
        swap_amount.0,
        "mcs balance of usdc doesn't change"
    );

    Ok(())
}
