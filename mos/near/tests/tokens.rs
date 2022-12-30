use near_sdk::json_types::U128;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::AccountId;

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn test_manage_mcs_token_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;
    for i in 0..2 {
        let token_account: AccountId = format!("eth_token{}.test.near", i).parse().unwrap();

        let to_chain: U128 = U128(i);
        let res = gen_call_transaction(
            &worker,
            &mcs,
            "add_mcs_token_to_chain",
            json!({"token": token_account, "to_chain": to_chain}),
            false,
        )
        .transact()
        .await;
        assert!(
            res.is_err(),
            "add_mcs_token_to_chain should fail since it is not registered"
        );

        let token = deploy_mcs_token_and_set_decimals(&worker, &mcs, &token_account, 24).await?;

        let res = gen_call_transaction(
            &worker,
            &mcs,
            "register_token",
            json!({"token": token_account, "mintable": true}),
            true,
        )
        .transact()
        .await?;
        assert!(res.is_success(), "register_token should succeed");

        let res = gen_call_transaction(
            &worker,
            &mcs,
            "add_mcs_token_to_chain",
            json!({"token": token_account, "to_chain": to_chain}),
            false,
        )
        .transact()
        .await?;
        assert!(
            res.is_success(),
            "add_mcs_token_to_chain should succeed since it has been deployed"
        );

        let is_valid = gen_call_transaction(
            &worker,
            &mcs,
            "valid_mcs_token_out",
            json!({"token": token_account, "to_chain": to_chain}),
            false,
        )
        .view()
        .await?
        .json::<bool>()?;
        assert!(
            is_valid,
            "mcs token {} to chain {} should be valid",
            token_account, i
        );

        let to_chain_2: U128 = U128(i + 1);
        let is_valid = gen_call_transaction(
            &worker,
            &mcs,
            "valid_mcs_token_out",
            json!({"token": token_account, "to_chain": to_chain_2}),
            false,
        )
        .view()
        .await?
        .json::<bool>()?;
        assert!(
            !is_valid,
            "mcs token {} to chain {} should be invalid",
            token_account,
            i + 1
        );

        let res = gen_call_transaction(
            &worker,
            &mcs,
            "remove_mcs_token_to_chain",
            json!({"token": token_account, "to_chain": to_chain}),
            false,
        )
        .transact()
        .await?;
        assert!(res.is_success(), "remove_mcs_token_to_chain should succeed");

        let is_valid = gen_call_transaction(
            &worker,
            &mcs,
            "valid_mcs_token_out",
            json!({"token": token_account, "to_chain": to_chain}),
            false,
        )
        .view()
        .await?
        .json::<bool>()?;
        assert!(
            !is_valid,
            "mcs token {} to chain {} should be invalid",
            token_account, i
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_manage_fungible_token_to_chain() -> anyhow::Result<()> {
    let worker = init_worker().await?;
    let mcs = prepare_mcs(&worker).await?;

    let mut token_name0 = "eth_token".to_string();
    let to_chain: U128 = U128(1);
    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name0, "to_chain": to_chain}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        !is_valid,
        "fungible token {} to chain {} should be invalid",
        token_name0, to_chain.0
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name0, "to_chain": to_chain}),
        false,
    )
    .transact()
    .await;
    assert!(
        res.is_err(),
        "add_fungible_token_to_chain should fail since the ft token does not exist"
    );

    let token_account: AccountId = "ft0.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;
    token_name0 = ft.id().to_string();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "register_token",
        json!({"token": token_name0, "mintable": false}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name0, "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed since it has been deployed"
    );

    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name0, "to_chain": to_chain}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        is_valid,
        "fungible token {} to chain {} should be valid",
        token_name0, to_chain.0
    );

    let to_chain_2: U128 = U128(to_chain.0 + 1);
    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name0, "to_chain": to_chain_2}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        !is_valid,
        "fungible token {} to chain {} should be invalid",
        token_name0, to_chain_2.0
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name0, "to_chain": to_chain_2}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed"
    );

    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name0, "to_chain": to_chain_2}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        is_valid,
        "fungible token {} to chain {} should be valid",
        token_name0, to_chain_2.0
    );

    let token_account: AccountId = "ft.test.near".parse().unwrap();
    let ft = deploy_and_init_ft(&worker, &token_account).await?;
    let token_name1 = ft.id().to_string();

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "register_token",
        json!({"token": token_name1, "mintable": false}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": token_name1, "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed since it has been deployed"
    );

    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name1, "to_chain": to_chain}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        is_valid,
        "fungible token {} to chain {} should be valid",
        token_name1, to_chain.0
    );

    let tokens = mcs
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<(String, Vec<U128>)>>()?;
    assert_eq!(2, tokens.len(), "wrong fungible tokens size");
    assert_eq!(
        token_name0,
        tokens.get(0).unwrap().0,
        "{} is not contained",
        token_name0
    );
    assert_eq!(
        token_name1,
        tokens.get(1).unwrap().0,
        "{} is not contained",
        token_name1
    );

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "remove_fungible_token_to_chain",
        json!({"token": token_name1, "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "remove_fungible_token_to_chain should succeed"
    );

    let is_valid = gen_call_transaction(
        &worker,
        &mcs,
        "valid_fungible_token_out",
        json!({"token": token_name1, "to_chain": to_chain}),
        false,
    )
    .view()
    .await?
    .json::<bool>()?;
    assert!(
        !is_valid,
        "fungible token {} to chain {} should be invalid",
        token_name1, to_chain.0
    );

    let tokens = mcs
        .call(&worker, "get_fungible_tokens")
        .view()
        .await?
        .json::<Vec<(String, Vec<U128>)>>()?;
    assert_eq!(1, tokens.len(), "wrong fungible tokens size");
    assert_eq!(
        token_name0,
        tokens.get(0).unwrap().0,
        "{} is not contained",
        token_name0
    );

    Ok(())
}
