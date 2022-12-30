use hex;
use near_sdk::json_types::U128;
use near_sdk::log;
// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::{prelude::*, AccountId};

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn test_deposit_out_native() -> anyhow::Result<()> {
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

    let to_chain: U128 = U128(MAP_CHAIN_ID);
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

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = parse_near!("10 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("log: {:?}", res.logs());
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_0:{}, balance_from_1:{}",
        balance_from_0,
        balance_from_1
    );
    log!(
        "balance_mcs_0:{}, balance_mcs_1:{}",
        balance_mcs_0,
        balance_mcs_1
    );
    log!(
        "{}, {}",
        balance_from_0 - balance_from_1,
        balance_mcs_1 - balance_mcs_0
    );
    assert!(balance_from_0 - balance_from_1 > amount);
    assert!(balance_mcs_1 - balance_mcs_0 < amount);

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
async fn test_deposit_out_native_invalid_account() -> anyhow::Result<()> {
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

    let to_chain: U128 = U128(MAP_CHAIN_ID);
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

    let from = worker.dev_create_account().await?;
    let to = hex::decode("abcd").unwrap();
    let amount: u128 = parse_near!("0.001 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("address length is incorrect for evm chain type"),
        "should be invalid address error"
    );
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_0:{}, balance_from_1:{}",
        balance_from_0,
        balance_from_1
    );
    log!(
        "balance_mcs_0:{}, balance_mcs_1:{}",
        balance_mcs_0,
        balance_mcs_1
    );

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_native_too_small() -> anyhow::Result<()> {
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

    let to_chain: U128 = U128(MAP_CHAIN_ID);
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

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = parse_near!("0.0009 N");
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("amount too small"),
        "should be amount too small error"
    );
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_0:{}, balance_from_1:{}",
        balance_from_0,
        balance_from_1
    );
    log!(
        "balance_mcs_0:{}, balance_mcs_1:{}",
        balance_mcs_0,
        balance_mcs_1
    );

    let amount: u128 = parse_near!("0.001 N");
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("logs: {:?}", res.logs());
    let balance_from_2 = from.view_account(&worker).await?.balance;
    let balance_mcs_2 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_2:{}, balance_from_1:{}",
        balance_from_2,
        balance_from_1
    );
    log!(
        "balance_mcs_2:{}, balance_mcs_1:{}",
        balance_mcs_2,
        balance_mcs_1
    );
    log!(
        "{}, {}",
        balance_from_1 - balance_from_2,
        balance_mcs_2 - balance_mcs_1
    );
    assert!(balance_from_1 - balance_from_2 > amount);

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
async fn test_deposit_out_native_no_deposit() -> anyhow::Result<()> {
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

    let to_chain: U128 = U128(MAP_CHAIN_ID);
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

    let from = worker.dev_create_account().await?;
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let amount: u128 = 0;
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let balance_mcs_0 = mcs.view_account(&worker).await?.balance;
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_native should fail");
    println!("error: {:?}", res.as_ref().err());
    assert!(
        res.err().unwrap().to_string().contains("amount should > 0"),
        "should be amount error"
    );
    let balance_from_1 = from.view_account(&worker).await?.balance;
    let balance_mcs_1 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_0:{}, balance_from_1:{}",
        balance_from_0,
        balance_from_1
    );
    log!(
        "balance_mcs_0:{}, balance_mcs_1:{}",
        balance_mcs_0,
        balance_mcs_1
    );
    log!(
        "{}, {}",
        balance_from_0 - balance_from_1,
        balance_mcs_1 - balance_mcs_0
    );
    assert!(balance_from_0 - balance_from_1 > amount);
    assert!(balance_mcs_1 - balance_mcs_0 > amount);
    assert!((balance_from_0 - balance_from_1 - amount) > (balance_mcs_1 - balance_mcs_0 - amount));
    log!("{}", balance_from_0 - balance_from_1 - amount);
    log!("{}", balance_mcs_1 - balance_mcs_0 - amount);

    let amount: u128 = parse_near!("1 N");
    let res = from
        .call(&worker, mcs.id(), "deposit_out_native")
        .args_json(json!({ "to": to }))?
        .gas(200_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "deposit_out_native should succeed");
    println!("logs: {:?}", res.logs());
    assert!(
        res.logs().get(2).unwrap().contains(DEPOSIT_OUT_TYPE),
        "should be deposit out log"
    );
    let balance_from_2 = from.view_account(&worker).await?.balance;
    let balance_mcs_2 = mcs.view_account(&worker).await?.balance;
    log!(
        "balance_from_2:{}, balance_from_1:{}",
        balance_from_2,
        balance_from_1
    );
    log!(
        "balance_mcs_2:{}, balance_mcs_1:{}",
        balance_mcs_2,
        balance_mcs_1
    );
    log!(
        "{}, {}",
        balance_from_1 - balance_from_2,
        balance_mcs_2 - balance_mcs_1
    );
    assert!(balance_from_1 - balance_from_2 > amount);
    assert!(balance_mcs_2 - balance_mcs_1 < amount);

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
async fn test_deposit_out_fungible_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

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

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
    assert!(
        res.logs().get(2).unwrap().contains(DEPOSIT_OUT_TYPE),
        "should be deposit out log"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after deposit out ft balance of root account is {:?}",
        balance
    );
    assert_eq!(
        total.0 - amount.0,
        balance.0,
        "after deposit out ft balance of from account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
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
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount":total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("abcd").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
async fn test_deposit_out_fungible_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount":total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, Some(18)).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount":total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );

    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
    assert_eq!(amount.0, balance.0, "after transfer out ft balance of mcs");
    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let to_chain: U128 = U128(100);
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = prepare_ft(&worker, &mcs, &token_account, to_chain, None).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &ft,
        "mint",
        json!({"account_id": from.id(), "amount":total}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint failed");
    println!("log: {:?}", res.logs());

    println!(
        "before deposit out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before deposit out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
        "after deposit out ft balance of from account is {:?}",
        balance
    );
    assert_eq!(
        total, balance,
        "after deposit out ft balance of root account"
    );

    let balance = ft
        .call(&worker, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_fungible_token_not_registered() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
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

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let msg = TokenReceiverMessage::Deposit { to };
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
async fn test_deposit_out_mcs_token() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before deposit out mcs token balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before deposit out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to, amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(2, res.logs().len(), "should have 3 logs");
    assert!(
        res.logs().get(1).unwrap().contains(DEPOSIT_OUT_TYPE),
        "should be deposit out log"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after deposit out ft balance of root account is {:?}",
        balance
    );
    assert_eq!(
        total.0 - amount.0,
        balance.0,
        "after deposit out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_no_1_yocto() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before deposit out mcs token balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before deposit out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;
    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to, amount))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    assert!(res.is_err(), "ft_transfer_call should fail");
    println!("ft_transfer_call error: {:?}", res.as_ref().err().unwrap());
    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"),
        "should be 1 yoctoNEAR error"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((from.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!(
        "after deposit out ft balance of root account is {:?}",
        balance
    );
    assert_eq!(
        total.0, balance.0,
        "after deposit out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after deposit out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after deposit out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_amount_too_small() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let amount: U128 = U128(1000000000000000000000 - 1);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to.clone(), amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_token should failed");
    assert!(
        res.as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("amount too small"),
        "should be amount too small error"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000000000);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to, amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(2, res.logs().len(), "should have 3 logs");
    assert!(
        res.logs().get(1).unwrap().contains(DEPOSIT_OUT_TYPE),
        "should be deposit out log"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_amount_too_large() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let amount: U128 = U128(total.0 + 1);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to.clone(), amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_token should failed");
    assert!(
        res.as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("burn mcs token or call near_deposit() failed"),
        "should be burn mcs token failed error"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_diff_decimal() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(MAP_CHAIN_ID);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 18).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let amount: U128 = U128(1000000000000000 - 1);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to.clone(), amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_token should failed");
    assert!(
        res.as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("amount too small"),
        "should be amount too small error"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    let amount: U128 = U128(1000000000000000);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to, amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");
    println!("ft_transfer_call logs: {:?}", res.logs());
    assert_eq!(2, res.logs().len(), "should have 3 logs");
    assert!(
        res.logs().get(1).unwrap().contains(DEPOSIT_OUT_TYPE),
        "should be deposit out log"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}

#[tokio::test]
async fn test_deposit_out_mcs_token_no_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let to_chain: U128 = U128(100);
    let token_account: AccountId = "mcs_token_0.test.near".parse().unwrap();
    let token = prepare_mos_token(&worker, &mcs, &token_account, to_chain, 24).await?;

    let from = worker.dev_create_account().await?;
    let total: U128 = U128::from(1000000000000000000000000000);
    let res = mcs
        .as_account()
        .call(&worker, &token_account, "mint")
        .args_json(json!({"account_id": from.id(), "amount": total}))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("3 N"))
        .transact()
        .await?;
    assert!(res.is_success(), "mint should succeed");
    println!("log: {:?}", res.logs());

    println!(
        "before transfer out ft balance of {} is {:?}",
        from.id(),
        total
    );
    println!("before transfer out ft balance of mcs is 0");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let balance_from_0 = from.view_account(&worker).await?.balance;

    let amount: U128 = U128(100000000000000000000000000);
    let res = from
        .call(&worker, &mcs.id(), "deposit_out_token")
        .args_json((token_account.to_string(), to.clone(), amount))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await;
    assert!(res.is_err(), "deposit_out_token should failed");
    assert!(
        res.as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("not supported"),
        "should be not supported error"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
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
        "after transfer out ft balance of from account"
    );

    let balance = from
        .call(&worker, &token_account, "ft_balance_of")
        .args_json((mcs.id().to_string(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("after transfer out ft balance of mcs is {:?}", balance);
    assert_eq!(0, balance.0, "after transfer out ft balance of mcs");

    Ok(())
}
