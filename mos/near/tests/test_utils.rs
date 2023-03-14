use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{serde, Balance};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::network::Sandbox;
use workspaces::operations::CallTransaction;
use workspaces::types::{KeyType, SecretKey};
use workspaces::{prelude::*, Account, AccountId, Contract, DevNetwork, Worker};

pub const MOCK_MAP_CLIENT_WASM_FILEPATH: &str =
    "./target/wasm32-unknown-unknown/release/mock_map_client.wasm";
pub const MULTISIG_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/multisig.wasm";
pub const MCS_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mos.wasm";
pub const MCS_TOKEN_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/mos_token.wasm";
pub const WNEAR_WASM_FILEPATH: &str = "./tests/data/w_near.wasm";
pub const REF_EXCHANGE_WASM_FILEPATH: &str = "./tests/data/ref_exchange.wasm";
pub const BUTTER_CORE_WASM_FILEPATH: &str = "./tests/data/butter_core.wasm";
pub const NEAR_SANDBOX_BIN_PATH: &str = "NEAR_SANDBOX_BIN_PATH";
pub const MAP_BRIDGE_ADDRESS: &str = "765a5a86411ab8627516cbb77d5db00b74fe610d";
pub const MAP_CHAIN_ID: u128 = 22776;
pub const DEV_ACCOUNT_SEED: &str = "testificate";
pub const TRANSFER_OUT_TYPE: &str =
    "2ef1cdf83614a69568ed2c96a275dd7fb2e63a464aa3a0ffe79f55d538c8b3b5";
pub const DEPOSIT_OUT_TYPE: &str =
    "150bd848adaf4e3e699dcac82d75f111c078ce893375373593cc1b9208998377";
pub const SWAP_OUT_TYPE: &str = "ca1cf8cebf88499429cca8f87cbca15ab8dafd06702259a5344ddce89ef3f3a5";
pub const ORDER_ID_DEPOSIT: Balance = 1640000000000000000000;

pub type Address = [u8; 20];

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ChainType {
    EvmChain,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum TokenReceiverMessage {
    Transfer {
        #[serde(with = "crate::hexstring")]
        to: Vec<u8>,
        to_chain: U128,
    },
    Deposit {
        #[serde(with = "crate::hexstring")]
        to: Vec<u8>,
    },
    Swap {
        #[serde(with = "crate::hexstring")]
        to: Vec<u8>,
        to_chain: U128,
        swap_info: SwapInfo,
    },
    LostFound {
        account: AccountId,
        is_native: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapInfo {
    pub src_swap: Vec<SwapParam>,
    pub dst_swap: SwapData,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapParam {
    pub amount_in: U128,
    pub min_amount_out: U128,
    #[serde(with = "crate::hexstring")]
    pub path: Vec<u8>,
    pub router_index: U64,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapData {
    pub swap_param: Vec<SwapParam>,
    #[serde(with = "crate::hexstring")]
    pub target_token: Vec<u8>,
    #[serde(with = "crate::hexstring")]
    pub map_target_token: Address,
}

pub fn gen_call_transaction<'a, U: serde::Serialize>(
    worker: &'a Worker<Sandbox>,
    contract: &'a Contract,
    function: &'a str,
    args: U,
    deposit: bool,
) -> CallTransaction<'a, 'a, impl DevNetwork> {
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

pub fn gen_call_transaction_by_account<'a, U: serde::Serialize>(
    worker: &'a Worker<Sandbox>,
    account: &Account,
    contract: &'a Contract,
    function: &'a str,
    args: U,
    deposit: bool,
) -> CallTransaction<'a, 'a, impl DevNetwork> {
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

pub fn new_account(account_id: &AccountId, sk: &SecretKey) -> Account {
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

pub async fn deploy_and_init_wnear(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let account_id: AccountId = "wrap.test.near".to_string().parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account_id.clone(), sk).await?.unwrap();
    let contract = account
        .deploy(&worker, &std::fs::read(WNEAR_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy wnear contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init WNEAR contract failed!");

    Ok(contract)
}

pub async fn deploy_mcs_token_and_set_decimals(
    worker: &Worker<Sandbox>,
    mcs: &Contract,
    token_account: &AccountId,
    decimals: u8,
) -> anyhow::Result<Contract> {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let contract = account
        .deploy(&worker, &std::fs::read(MCS_TOKEN_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy mos token contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({"controller": contract.id(), "owner": contract.id()}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new mos token contract failed!");

    let res = gen_call_transaction(
        worker,
        &contract,
        "set_metadata",
        json!({"address": contract.id(), "decimals": decimals}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "set_metadata for {} failed",
        contract.id()
    );

    let res = gen_call_transaction(
        worker,
        &contract,
        "set_controller",
        json!({"controller": mcs.id()}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "set_controller for {} failed",
        contract.id()
    );

    Ok(contract)
}

pub async fn prepare_mos_token(
    worker: &Worker<Sandbox>,
    mcs: &Contract,
    token_account: &AccountId,
    to_chain: U128,
    decimals: u8,
) -> anyhow::Result<Contract> {
    let token = deploy_mcs_token_and_set_decimals(&worker, mcs, token_account, decimals).await?;

    let res = gen_call_transaction(
        &worker,
        mcs,
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
        json!({"token": token.id().to_string(), "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_mcs_token_to_chain should succeed");

    Ok(token)
}

pub async fn deploy_and_init_ft(
    worker: &Worker<Sandbox>,
    token_account: &AccountId,
) -> anyhow::Result<Contract> {
    deploy_and_init_ft_with_decimal(worker, token_account, 24).await
}

pub async fn deploy_and_init_ft_with_decimal(
    worker: &Worker<Sandbox>,
    token_account: &AccountId,
    decimal: u8,
) -> anyhow::Result<Contract> {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();

    let contract = account
        .deploy(&worker, &std::fs::read(MCS_TOKEN_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy ft contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({"controller": contract.id(), "owner": contract.id()}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init ft contract failed!");

    let res = gen_call_transaction(
        worker,
        &contract,
        "set_metadata",
        json!({"address": contract.id(), "decimals": decimal}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "set_metadata for {} failed",
        contract.id()
    );

    Ok(contract)
}

pub async fn prepare_ft(
    worker: &Worker<Sandbox>,
    mcs: &Contract,
    token_account: &AccountId,
    to_chain: U128,
    decimals_opt: Option<u8>,
) -> anyhow::Result<Contract> {
    let ft = if let Some(decimals) = decimals_opt {
        deploy_and_init_ft_with_decimal(&worker, token_account, decimals).await?
    } else {
        deploy_and_init_ft(&worker, token_account).await?
    };

    let res = gen_call_transaction(
        &worker,
        mcs,
        "register_token",
        json!({"token": ft.id().to_string(), "mintable": false}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "register_token should succeed");

    let res = gen_call_transaction(
        &worker,
        &mcs,
        "add_fungible_token_to_chain",
        json!({"token": ft.id().to_string(), "to_chain": to_chain}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "add_fungible_token_to_chain should succeed"
    );

    Ok(ft)
}

pub async fn deploy_and_init_mcs(
    worker: &Worker<Sandbox>,
    map_light_client: String,
    map_bridge_address: String,
    wrapped_token: String,
) -> anyhow::Result<Contract> {
    let token_account: AccountId = "mcs.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let contract = account
        .deploy(&worker, &std::fs::read(MCS_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy mcs contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "init")
        .args_json(json!({"owner": contract.id(),
            "map_light_client": map_light_client,
            "map_bridge_address":map_bridge_address,
            "wrapped_token": wrapped_token,
            "near_chain_id": "1313161555",
        "map_chain_id": "22776",
            "ref_exchange": "ref.near",
            "butter_core": []}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    Ok(contract)
}

pub async fn prepare_mcs(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let map_client = deploy_and_init_light_client(&worker).await?;
    let wnear = deploy_and_init_wnear(&worker).await?;
    deploy_and_init_mcs(
        &worker,
        map_client.id().to_string(),
        MAP_BRIDGE_ADDRESS.to_string(),
        wnear.id().to_string(),
    )
    .await
}

pub async fn prepare_mcs_with_ref_and_core(
    worker: &Worker<Sandbox>,
    wnear: &Contract,
    ref_exchange: &Contract,
    cores: Vec<AccountId>,
) -> anyhow::Result<Contract> {
    let map_client = deploy_and_init_light_client(&worker).await?;

    let token_account: AccountId = "mcs.test.near".parse().unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(token_account.clone(), sk).await?.unwrap();
    let mcs = account
        .deploy(&worker, &std::fs::read(MCS_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy mcs contract id: {:?}", mcs.id());

    for core in cores.clone() {
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        let core_account = worker.create_tla(core, sk).await?.unwrap();
        let contract = core_account
            .deploy(&worker, &std::fs::read(BUTTER_CORE_WASM_FILEPATH)?)
            .await?
            .unwrap();
        println!("deploy butter core contract id: {:?}", contract.id());

        let res = contract
            .call(&worker, "new")
            .args_json(json!({
                "controller": mcs.id(),
                "ref_exchange":ref_exchange.id(),
                "wrapped_token": wnear.id(),
                    "owner": contract.id()
            }))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success(), "new ref exchange contract failed!");
    }

    let res = mcs
        .call(&worker, "init")
        .args_json(json!({"owner": mcs.id(),
            "map_light_client": map_client.id(),
            "map_bridge_address":MAP_BRIDGE_ADDRESS.to_string(),
            "wrapped_token": wnear.id(),
            "near_chain_id": "1313161555",
        "map_chain_id": "22776",
            "ref_exchange": ref_exchange.id(),
            "butter_core": cores}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init MCS contract failed!");
    println!("init mcs logs: {:?}", res.logs());

    Ok(mcs)
}

pub async fn deploy_and_init_light_client(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let contract = worker
        .dev_deploy(&std::fs::read(MOCK_MAP_CLIENT_WASM_FILEPATH)?)
        .await?;
    println!("deploy map light client contract id: {:?}", contract.id());

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    let res = contract
        .call(&worker, "new")
        // .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    Ok(contract)
}

pub async fn deploy_token_with_account(
    worker: &Worker<Sandbox>,
    account: AccountId,
) -> anyhow::Result<Contract> {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account = worker.create_tla(account, sk).await?.unwrap();
    let contract = account
        .deploy(&worker, &std::fs::read(MCS_TOKEN_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy token contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({"controller": contract.id(), "owner": contract.id()}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "new contract failed!");

    let res = gen_call_transaction(
        worker,
        &contract,
        "set_metadata",
        json!({ "decimals": 24}),
        false,
    )
    .transact()
    .await?;
    assert!(
        res.is_success(),
        "set_metadata for {} failed",
        contract.id()
    );

    Ok(contract)
}

pub async fn prepare_ref_exchange(
    worker: &Worker<Sandbox>,
    ref_exchange: AccountId,
    owner: &Account,
) -> anyhow::Result<Contract> {
    println!("start deploying ref exchange contract");
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let ref_account = worker.create_tla(ref_exchange, sk).await?.unwrap();
    let contract = ref_account
        .deploy(&worker, &std::fs::read(REF_EXCHANGE_WASM_FILEPATH)?)
        .await?
        .unwrap();
    println!("deploy ref exchange contract id: {:?}", contract.id());

    let res = contract
        .call(&worker, "new")
        .args_json(json!({
            "owner_id": owner.id(),
            "exchange_fee":4,
            "referral_fee": 1
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success(), "new ref exchange contract failed!");

    Ok(contract)
}

pub async fn add_ref_exchange_pool(
    worker: &Worker<Sandbox>,
    ref_exchange: &Contract,
    owner: &Account,
    token0: &Contract,
    token1: &Contract,
    amount: U128,
) -> anyhow::Result<u64> {
    let res = gen_call_transaction_by_account(
        worker,
        &owner,
        &ref_exchange,
        "add_simple_pool",
        json!({"tokens": [token0.id(), token1.id()], "fee": 25}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_simple_pool failed");
    let pool_id: u64 = res.json()?;

    println!("5.0");
    let res = gen_call_transaction_by_account(
        worker,
        &owner,
        &ref_exchange,
        "storage_deposit",
        json!({}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "storage_deposit failed");

    println!("5.1");
    let res = owner
        .call(&worker, &ref_exchange.id(), "register_tokens")
        .args_json(json!({"token_ids": [token0.id(), token1.id()]}))
        .unwrap()
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "register_tokens failed");

    println!("5.2");
    let res = gen_call_transaction_by_account(
        worker,
        &owner,
        &token0,
        "storage_deposit",
        json!({"account_id": ref_exchange.id()}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "storage_deposit failed");

    println!("5.3");
    let res = gen_call_transaction_by_account(
        worker,
        &owner,
        &token1,
        "storage_deposit",
        json!({"account_id": ref_exchange.id()}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "storage_deposit failed");

    println!("5.4");
    let res = owner
        .call(&worker, &token0.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id": ref_exchange.id(),
        "amount":amount,
        "msg": ""}))
        .unwrap()
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");

    println!("5.5");
    let res = owner
        .call(&worker, &token1.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id": ref_exchange.id(),
        "amount":amount,
        "msg": ""}))
        .unwrap()
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success(), "ft_transfer_call failed");

    println!("5.6");
    let res = gen_call_transaction_by_account(
        worker,
        &owner,
        &ref_exchange,
        "add_liquidity",
        json!({"pool_id": pool_id,
        "amounts":[amount,amount]}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "add_liquidity failed");

    println!("5.7");
    Ok(pool_id)
}

pub async fn near_deposit(
    worker: &Worker<Sandbox>,
    wnear: &Contract,
    user: &Account,
    amount: u128,
) -> anyhow::Result<()> {
    let res = gen_call_transaction_by_account(
        &worker,
        &user,
        &wnear,
        "storage_deposit",
        json!({"account_id": user.id()}),
        true,
    )
    .transact()
    .await?;
    assert!(res.is_success(), "storage_deposit failed");

    let res = user
        .call(&worker, &wnear.id(), "near_deposit")
        .args_json(json!({}))
        .unwrap()
        .gas(300_000_000_000_000)
        .deposit(amount)
        .transact()
        .await?;
    assert!(res.is_success(), "near deposit failed");

    Ok(())
}

pub async fn init_worker() -> anyhow::Result<Worker<Sandbox>> {
    std::env::var(NEAR_SANDBOX_BIN_PATH)
        .expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::sandbox().await?;

    Ok(worker)
}

pub(crate) mod hexstring {
    use hex::FromHex;
    use near_sdk::serde::{de::Error, Deserialize, Deserializer, Serializer};

    /// Deserialize string into T
    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: hex::FromHex,
        <T as FromHex>::Error: std::fmt::Display,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s.len() <= 2 || !s.starts_with("0x") {
            return T::from_hex(Vec::new()).map_err(D::Error::custom);
        }

        T::from_hex(&s[2..]).map_err(D::Error::custom)
    }

    /// Serialize from T into string
    pub(crate) fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let hex_string = hex::encode(value.as_ref());
        if hex_string.is_empty() {
            return serializer.serialize_str("");
        }

        serializer.serialize_str(&(String::from("0x") + &hex_string))
    }
}
