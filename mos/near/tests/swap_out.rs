use hex;
use near_sdk::json_types::{U128, U64};
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
async fn test_swap_out_mcs_token() -> anyhow::Result<()> {
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

    let pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

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

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);
    let res = gen_call_transaction_by_account(
        &worker,
        &mcs.as_account(),
        &usdc,
        "mint",
        json!({"account_id": user.id(), "amount": swap_amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let token_recevier_msg = TokenReceiverMessage::Swap {
        to,
        to_chain,
        swap_info: SwapInfo {
            src_swap: vec![SwapParam {
                amount_in: U128(0),
                min_amount_out: U128(1000000000000000000000000),
                path: "usdc.test.nearXwrap.test.near".as_bytes().to_vec(),
                router_index: U64(pool_id),
            }],
            dst_swap: SwapData {
                swap_param: vec![],
                target_token: vec![1; 20],
                map_target_token: [0; 20],
            },
        },
    };

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap out: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = user
        .call(&worker, usdc.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id": mcs.id(), "amount": swap_amount, "msg": serde_json::to_string(&token_recevier_msg).unwrap()}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("logs {:?}", res.logs());
    assert!(
        res.logs().last().unwrap().contains(SWAP_OUT_TYPE),
        "can not get swap out log"
    );
    println!("{}", res.logs().last().unwrap());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after transfer out: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    assert!(mcs_balance_1 > mcs_balance_0, "mcs balance increase");
    assert!(user_balance_0 > user_balance_1, "user balance decrease");

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of mcs is incorrect");

    let mcs_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_1);
    assert!(mcs_balance_wnear_1 > mcs_balance_wnear_0);

    Ok(())
}

#[tokio::test]
async fn test_swap_out_mcs_token_min_mount_out_too_big() -> anyhow::Result<()> {
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

    let pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

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

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);
    let res = gen_call_transaction_by_account(
        &worker,
        &mcs.as_account(),
        &usdc,
        "mint",
        json!({"account_id": user.id(), "amount": swap_amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");

    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let token_recevier_msg = TokenReceiverMessage::Swap {
        to,
        to_chain,
        swap_info: SwapInfo {
            src_swap: vec![SwapParam {
                amount_in: U128(0),
                min_amount_out: U128(3000000000000000000000000),
                path: "usdc.test.nearXwrap.test.near".as_bytes().to_vec(),
                router_index: U64(pool_id),
            }],
            dst_swap: SwapData {
                swap_param: vec![],
                target_token: vec![1; 20],
                map_target_token: [0; 20],
            },
        },
    };

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap out: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = user
        .call(&worker, usdc.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id": mcs.id(), "amount": swap_amount, "msg": serde_json::to_string(&token_recevier_msg).unwrap()}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("logs {:?}", res.logs());
    assert!(
        !res.logs().last().unwrap().contains(SWAP_OUT_TYPE),
        "should not get swap out log"
    );

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after transfer out: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    assert!(mcs_balance_1 > mcs_balance_0, "mcs balance increase");
    assert!(user_balance_0 > user_balance_1, "user balance decrease");

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        swap_amount,
        balance,
        "balance of {} is incorrect",
        user.id()
    );

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of mcs is incorrect");

    let mcs_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_1);
    assert_eq!(mcs_balance_wnear_1, mcs_balance_wnear_0);

    Ok(())
}

#[tokio::test]
async fn test_swap_out_ft_token() -> anyhow::Result<()> {
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

    let pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

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

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);
    let res = gen_call_transaction(
        &worker,
        &usdc,
        "mint",
        json!({"account_id": user.id(), "amount": swap_amount}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "mint usdc failed");

    println!("start to swap out");
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let token_recevier_msg = TokenReceiverMessage::Swap {
        to,
        to_chain,
        swap_info: SwapInfo {
            src_swap: vec![SwapParam {
                amount_in: U128(0),
                min_amount_out: U128(1000000000000000000000000),
                path: "usdc.test.nearXwrap.test.near".as_bytes().to_vec(),
                router_index: U64(pool_id),
            }],
            dst_swap: SwapData {
                swap_param: vec![],
                target_token: vec![1; 20],
                map_target_token: [0; 20],
            },
        },
    };

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap out: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = user
        .call(&worker, usdc.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id": mcs.id(), "amount": swap_amount, "msg": serde_json::to_string(&token_recevier_msg).unwrap()}))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call should succeed");
    println!("logs {:?}", res.logs());
    assert!(
        res.logs().last().unwrap().contains(SWAP_OUT_TYPE),
        "can not get swap out log"
    );
    println!("{}", res.logs().last().unwrap());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after transfer out: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    assert!(mcs_balance_1 > mcs_balance_0, "mcs balance increase");
    assert!(user_balance_0 > user_balance_1, "user balance decrease");

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of mcs is incorrect");

    let mcs_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_1);
    assert!(mcs_balance_wnear_1 > mcs_balance_wnear_0);

    Ok(())
}

#[tokio::test]
async fn test_swap_out_native_token() -> anyhow::Result<()> {
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

    let pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

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

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let swap_info = SwapInfo {
        src_swap: vec![SwapParam {
            amount_in: U128(0),
            min_amount_out: U128(1000000000000000000000000),
            path: "wrap.test.nearXusdc.test.near".as_bytes().to_vec(),
            router_index: U64(pool_id),
        }],
        dst_swap: SwapData {
            swap_param: vec![],
            target_token: vec![1; 20],
            map_target_token: [0; 20],
        },
    };

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap out: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = user
        .call(&worker, mcs.id(), "swap_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain, "swap_info": swap_info}))?
        .gas(300_000_000_000_000)
        .deposit(swap_amount.0)
        .transact()
        .await?;
    assert!(res.is_success(), "swap_out_native should succeed");
    println!("logs {:?}", res.logs());
    assert!(
        res.logs().last().unwrap().contains(SWAP_OUT_TYPE),
        "can not get swap out log"
    );
    println!("{}", res.logs().last().unwrap());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after transfer out: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    assert!(mcs_balance_1 > mcs_balance_0, "mcs balance increase");
    assert!(
        user_balance_0 - swap_amount.0 > user_balance_1,
        "user balance decrease"
    );

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("usdc balance of mcs: {:?}", balance);
    assert!(balance.0 > 0, "balance of mcs is incorrect");

    let user_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of user is {:?}", user_balance_wnear_1);
    assert_eq!(0, user_balance_wnear_1.0);

    let mcs_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_1);
    assert_eq!(0, mcs_balance_wnear_1.0);

    Ok(())
}

#[tokio::test]
async fn test_swap_out_native_token_min_amount_out_too_big() -> anyhow::Result<()> {
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

    let pool_id =
        add_ref_exchange_pool(&worker, &ref_exchange, &owner, &usdc, &wnear, amount).await?;

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

    let account_id: AccountId = "pandarr.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let user = worker.create_tla(account_id.clone(), sk).await?.unwrap();

    let swap_amount: U128 = U128(3000000000000000000000000);
    let to = hex::decode("0f5Ea0A652E851678Ebf77B69484bFcD31F9459B").unwrap();
    let swap_info = SwapInfo {
        src_swap: vec![SwapParam {
            amount_in: U128(0),
            min_amount_out: U128(3000000000000000000000000),
            path: "wrap.test.nearXusdc.test.near".as_bytes().to_vec(),
            router_index: U64(pool_id),
        }],
        dst_swap: SwapData {
            swap_param: vec![],
            target_token: vec![1; 20],
            map_target_token: [0; 20],
        },
    };

    let mcs_balance_0 = mcs.view_account(&worker).await?.balance;
    let user_balance_0 = user.view_account(&worker).await?.balance;
    println!(
        "before swap out: mcs balance: {}, user balance: {} ",
        mcs_balance_0, user_balance_0
    );

    let mcs_balance_wnear_0 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_0);

    let res = user
        .call(&worker, mcs.id(), "swap_out_native")
        .args_json(json!({"to": to, "to_chain": to_chain, "swap_info": swap_info}))?
        .gas(300_000_000_000_000)
        .deposit(swap_amount.0)
        .transact()
        .await;
    assert!(res.is_err(), "swap_out_native should fail");
    println!("err {:?}", res.err().unwrap());

    let mcs_balance_1 = mcs.view_account(&worker).await?.balance;
    let user_balance_1 = user.view_account(&worker).await?.balance;
    println!(
        "after transfer out: mcs balance: {}, user balance: {}",
        mcs_balance_1, user_balance_1
    );

    assert!(mcs_balance_1 > mcs_balance_0, "mcs balance increase");
    assert!(
        user_balance_0 < swap_amount.0 + user_balance_1,
        "user balance decrease"
    );

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(0, balance.0, "balance of {} is incorrect", user.id());

    let balance = user
        .call(&worker, &usdc.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("usdc balance of mcs: {:?}", balance);
    assert_eq!(0, balance.0, "balance of mcs is incorrect");

    let user_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((user.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of user is {:?}", user_balance_wnear_1);
    assert_eq!(0, user_balance_wnear_1.0);

    let mcs_balance_wnear_1 = user
        .call(&worker, &wnear.id(), "ft_balance_of")
        .args_json((mcs.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("wrap balance of mcs is {:?}", mcs_balance_wnear_1);
    assert_eq!(0, mcs_balance_wnear_1.0);

    Ok(())
}
