use std::fs;
use near_sdk::json_types::U64;
use serde_json::json;
use workspaces::{prelude::*, Worker, Contract};
use workspaces::network::Sandbox;
use map_light_client::{EpochRecord, Validator};

const MAP_CLIENT_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/map_light_client.wasm";
const NEAR_SANDBOX_BIN_PATH: &str = "NEAR_SANDBOX_BIN_PATH";

/*
0
bls priv key:  0x230542ab27067b13089165757c4dfb02f67440c5b8888994de33b015bb07a142
g2 pub key:  0x2d692ebfd5b28f869cf87b12688504f1fd2194ad68d0bcbdde5f03ec45e98ef82e4114565378770ff9b81cc4488bbe93ba4dfaadf7a54c088560397588c1ab7c0de7dc40658ca64443100d757e9236555e7e1929edc3f398fa508ab5926bf1510eeabadcda0e475fcc4274349bdabdbf3c3855cc37548e2ebf7b0314436bee29
g1 pub key:  0x285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e3
secp256k1 priv key:  0x5655df9f81c8e045cf86d2e2e77778b9fa952c46391c29c41c96dedc533a64ad
secp256k1 pub key:  0x04d7aed26a1bf17e3ae412b24b194f4f4150066e963b99bad2361a149b80878c1f4574b3ecf5cacf3a258291eed55c19ef5fc38a084ef22d05dddf943ddf4f9463
secp256k1 address:  0x908D0FDaEAEFbb209BDcb540C2891e75616154b3
1
bls priv key:  0x23ee418be99759ba1a95389c47e21eccbaea41cc73261775ba7813dd6746e97c
g2 pub key:  0x2d12b2b13b2fc35845b0a7a4b1c4b794457ebe4dc176e9ec258a8b636d785623179b51bff3e3cb9ead360751665a14c238d15e828a6825b8c0d4f3b9a05837f006900ef3770f0f05ea7477415c98ac6182212f0d88dcc6b7f1b0cea61fd4b5bd1283e94132d5912258b0f1675056afafb71e33979df7c1776517ba37da38dadb
g1 pub key:  0x0d570979e84f504247c0ab6c1bc98967a300192132707a6d144cce74d77ab11a28feb22d09573a136a1ae43f0329f77be54968035d7b29161de64068b52fa0fb
secp256k1 priv key:  0x0a5f9842b46058800bec478523aa6e1141f592924ecda0d22074daf89045f396
secp256k1 pub key:  0x04139a44005d256d8b16da81b6caae548677814a514b592a68b10033276b353899f925efdaf78b1becaa8a905a57158ced6d1c9fa20c04529c63770f99e8858123
secp256k1 address:  0xEbf0E9FbC6210F199d1C34f2418b64129e7FF78A
2
bls priv key:  0x076a163412db5112c2de534976dc6f1cc4d3b1f1e5c1afa4e4a34e2aea0a14d5
g2 pub key:  0x1041c19ccd878527886aff2e70a2f07db76c3fa3155823677abb551bd480a5f329b1f7ea9a3aa75613faf865f960acad8e04bb661a8b8e87e48d172c83fd4c2b2255c47c8e92e5329ee42033153f8c9750ab5042f6d89f27add64e0d5fba36c623789ad2fdcbc68dd30d19c33ec60cde4d431411aa7ec754038c2fa8ffb41db4
g1 pub key:  0x06123bea2fdc5ca96f7b3810d7abb489bd04fa3db73487e82261bb0a768d96861958a18770f574432dd56665f8214ee885780c00de9e47f8a20e7b7075fa2448
secp256k1 priv key:  0xdbf57d88561e0f53c227b2229c4b3fb2bf36bda0e1a852a902cc421c8c5a787d
secp256k1 pub key:  0x04e55d879826dfedf134f1bd2043130d90dc3055ed564bc11c2f69c4624f2908ca07b322598c9a8666cd4532381ced27d67c23e4df3aed8f890d4aeabbb7f98dba
secp256k1 address:  0x8f189338912AC69AB776318A32Ad7473731a955F
3
bls priv key:  0x05f0159fe779e38e0ce377c02e5923826db71683b29628d96467dfcdb569158b
g2 pub key:  0x1b9fe96534980d89088570afeb7c6d1bb7db8c72df20e8f6a4ab2f4fde12e68609ce26b976e80794470dd62ed18e9a5402f3f0382aac051ac03f40478dc40a8d297b4dcc2c5edf9accd55a726bff6dd646adac58220c0f51dd0d0c7a66ffab870a1f48d542f5c97d9b6049d4a49f408612d6d1ce811bae95c68573cdc1ba648b
g1 pub key:  0x055c69baeedb58db6e1467eb7d1d51347ffe8bc9e30be2cf8638d8bf9b9b9a531c13d30bd973eabbc38c87b8ca3a846db126fcf62bfebe8516ca0b26f959b9ff
secp256k1 priv key:  0xd89e9314e35b989c6c9d0889faf0fcfe81005d4c589744ec58fd394bc37ad2d2
secp256k1 pub key:  0x040bfff54b4427902673b815426d0fbc375d613d252fa7865452f91f150f5e245aeff786e2b48c016d6ed26ce8284f0e899a5513c433f9d142a1727754cc9ca91a
secp256k1 address:  0xD762eD84dB64848366E74Ce43742C1960Ba62304
 */

const INIT_VALUE: &str = r#"{
            "threshold": "3",
            "validators":
                [
                    {
                        "g1_pub_key":{"x":"0x285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae","y":"0x218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e3"},
                        "address":"0x908D0FDaEAEFbb209BDcb540C2891e75616154b3",
                        "weight": "1"
                    },
                    {
                        "g1_pub_key":{"x":"0x0d570979e84f504247c0ab6c1bc98967a300192132707a6d144cce74d77ab11a","y":"0x28feb22d09573a136a1ae43f0329f77be54968035d7b29161de64068b52fa0fb"},
                        "address":"0xEbf0E9FbC6210F199d1C34f2418b64129e7FF78A",
                        "weight": "1"
                    },
                    {
                        "g1_pub_key":{"x":"0x06123bea2fdc5ca96f7b3810d7abb489bd04fa3db73487e82261bb0a768d9686","y":"0x1958a18770f574432dd56665f8214ee885780c00de9e47f8a20e7b7075fa2448"},
                        "address":"0x8f189338912AC69AB776318A32Ad7473731a955F",
                        "weight": "1"
                    },
                    {
                        "g1_pub_key":{"x":"0x055c69baeedb58db6e1467eb7d1d51347ffe8bc9e30be2cf8638d8bf9b9b9a53","y":"0x1c13d30bd973eabbc38c87b8ca3a846db126fcf62bfebe8516ca0b26f959b9ff"},
                        "address":"0xD762eD84dB64848366E74Ce43742C1960Ba62304",
                        "weight": "1"
                    }
                ],
            "epoch":"1",
            "epoch_size":"1000"
        }"#;

const HEADER_0_012: &str = r#"{
                "parentHash":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7",
                "coinbase":"0x908D0FDaEAEFbb209BDcb540C2891e75616154b3",
                "root":"0xecc60e00b3fe5ce9f6e1a10e5469764daf51f1fe93c22ec3f9a7583a80357217",
                "txHash":"0xd35d334d87c0cc0a202e3756bf81fae08b1575f286c7ee7a3f8df4f0f3afc55d",
                "receiptHash":"0x209e5a7f764f4adb03b2799a8ba555694806cd4d8f593776e263fd90f0630f42",
                "bloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "number":"0x3e8",
                "gasLimit":"0x0",
                "gasUsed":"0x5208",
                "time":"0x5c47775c",
                "extra":"0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b840276d8fe5533b4d570fbe9bc0b104022f9c67f5210d493c9558aef517db0713e0255250efd11d89993aac000054bcebd3bea74665a163030555bfd0ad01d8bedf01c3808080",
                "minDigest":"0x0000000000000000000000000000000000000000000000000000000000000000",
                "nonce":"0x0000000000000000",
                "baseFee":"0x0"
            }"#;
const AGG_PK_012: &str = r#"{
               "xi": "0x20e9a44d0a1efeb2d64bc51a487b938070a93985b02b43620360fc97239b2fbc",
                "xr": "0x12c716655a20451e8ad32df8244e203c8fee447d910af1c3b0da9a3f36b8ef8a",
                "yi": "0x2ce0db8fa40ea88590efad6a39cb8e944f81485be32dbd59946da3cc4d5de8c3",
                "yr": "0x11f69118a10952fb9800d0272b5ad4b8e36649843380cd043589df944956dfb8"
            }"#;
const AGG_PK_12: &str = r#"{
               "xi": "0x0739d7e0990d4462404afb7abeebf0c3f93f49affc75eaa8b7cbb03e60f49b9d",
                "xr": "0x1e1bc6b9ebe8c412975d78b8681d294579ab4c248058657952b9661101de0cd2",
                "yi": "0x2d826142d06bdfeaaf66dec55a4b0b52b1384ab889928ff63a43b5ced40737ed",
                "yr": "0x257f8e942ad9cf3569cc43f4ea39b2a7b2fe2f0e4cc1992f9a1d3d5dc3658ef5"
            }"#;
const AGG_PK_01: &str = r#"{
               "xi": "0x0e9462421703b53a5112487313ac2c1a48c886453190c57b445b622afe918aeb",
                "xr": "0x1d65a1b84c31d19a0596dc5df68a8ee3bff4640ba973bc90ea91333196116891",
                "yi": "0x05f3fae8626e70a4efe32383f3dc72b8c268ee04e6f0aadd61114a1d9e1d7869",
                "yr": "0x1f361674d6ecd3a8b689cdcda430e7df580d478d87795f0bfb71f7d660667f2d"
            }"#;

#[tokio::test]
async fn test_initialize() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let res = contract
        .call(&worker, "initialized")
        .args_json(json!({}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "call initialized fail");
    let result: bool = res.json()?;
    assert!(!result, "contract should not be initialized");

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let res = contract
        .call(&worker, "initialized")
        .args_json(json!({}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "call initialized fail");
    let result: bool = res.json()?;
    assert!(result, "contract should be initialized");

    Ok(())
}

#[tokio::test]
async fn test_update_block_header() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header failed");

    Ok(())
}

#[tokio::test]
async fn test_update_block_header_bad_number() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    header["number"] = json!("0x3e9");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    assert!(res.err().unwrap().to_string().contains("block header height is incorrect"), "unexpected failure reason");

    Ok(())
}

#[tokio::test]
async fn test_update_block_header_bad_epoch() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header should succeed");

    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");

    Ok(())
}

#[tokio::test]
async fn test_update_block_bad_ecdsa_signer() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    // use extra with ecdsa signature signed by another validator
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841b166f38dd2c80eaf82c185ae6fbfb3c4ad34eb24e784e2277f9427540b9d066804868983df05e962c72cc43e3f865a0d4e36eeef08ac4a5dcea2d286ba79540e01f84407b8400aa853e8c8f65c21d9722a08b169376ebbad4eb13d210f563b6759a14bc8361f173ccc5db2a85c10e7e58b51bb90cd5da0b2120b0b8d899624268f66882d452c01c3808080");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    assert!(res.err().unwrap().to_string().contains("ecdsa signer is not correct"), "unexpected failure reason");

    Ok(())
}

#[tokio::test]
async fn test_update_block_bad_threshold() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_01).unwrap();
    // use extra with agg seal signed by validator 01
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841abb150fcf44735d6df641c0b3f29f3a6b5088542031734e70ab48beec303aaff4e0a56372af7774be1df2240916de4b329dd747b38b53e258ff79af6ac93f3d901f84403b84018d5eb4a531049c047c9003f6c7dd9b30e9d7183bbb71d687836d863b24408471b338f2fcfa2ac4b21e1fa3447a82e8353b6513d458cdcc4a6a15d74b5cc6f0801c3808080");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    assert!(res.err().unwrap().to_string().contains("threshold is not satisfied"), "unexpected failure reason");

    Ok(())
}

#[tokio::test]
async fn test_update_block_bad_agg_pk() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_01).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    assert!(res.err().unwrap().to_string().contains("check g2 pub key failed"), "unexpected failure reason");

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_single_receipt() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success());

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    header["receiptHash"] = json!("0xc502fb6c3ccb075c3e4425885ce26c3b00dba0cf86f4016abcc375eb79dedfab");
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b84151bcd0f46fa9ec5d0d8ba37741e6336b7bf3c4de7077121f86f36c007690a5fe21db3dc88866bfd4a7455242f7ad7bf5e8484c5376d7495aac7b7c07b8ebfe2f00f84407b8402f7688147b3ee1e630e16b891d9ec9c8ab7c84021f9973877fd09493ef1925e81dd8906dcc2686feb95ae438c63b1009fe75f54d43dd7cb32ed5d61f9438b85f01c3808080");
    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({
            "receipt_proof" : {
            "header": header,
            "agg_pk": agg_pk,
            "receipt": {
                    "receipt_type": "1",
                    "post_state_or_status": "0x00",
                    "cumulative_gas_used": "2000",
                    "bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "logs": [{
                        "address": "0x0000000000000000000000000000000000000000",
                        "topics": ["0x0000000000000000000000000000000000000000000000000000000000000000"],
                        "data": "0x00"}]
            },
                "key_index":"0x80",
                "proof":["0xf9014d822080b9014701f90143008207d0b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f83af838940000000000000000000000000000000000000000e1a0000000000000000000000000000000000000000000000000000000000000000000"]
        }
,
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success());

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_multi_receipt() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({
            "receipt_proof" : {
            "header": header,
            "agg_pk":agg_pk,
            "receipt": {
                    "receipt_type": "1",
                    "post_state_or_status": "0x01",
                    "cumulative_gas_used": "2000",
                    "bloom": "0x01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "logs": [{
                        "address": "0x0100000000000000000000000000000000000000",
                        "topics": ["0x0000000000000000000000000000000000000000000000000000000000000001", "0x000000000000000000000000000000000000000000000000000000000000000a"],
                        "data": "0x01"}]
            },
                "key_index":"0x01",
                "proof":["0xf851a0141842b8380e7bf9240f0d0e46856a2675b85df570e95bdceec55b495629d4ca80808080808080a0a0a3ab578877d4702cfb44ffa01ae96b2365aacbe55715c8d6222174233dd6208080808080808080",
                "0xf85180a01ee4849328db1b6425ffeb9c54f70604fa6e855a7d0d55a0471645e14ab7cae9a0275c5d8aa15b6a2f8b17619ce74fbf1bc92d11f0bae44f226b14dde1a603e25a8080808080808080808080808080",
                "0xf9016d20b9016901f90165018207d0b9010001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f85cf85a940100000000000000000000000000000000000000f842a00000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000a01"]
        }
,
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "verify_proof_data failed");

    Ok(())
}

#[tokio::test]
async fn test_validator_remove_01() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    // remove validator 0
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b840276d8fe5533b4d570fbe9bc0b104022f9c67f5210d493c9558aef517db0713e0255250efd11d89993aac000054bcebd3bea74665a163030555bfd0ad01d8bedf01c3808080");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header failed");
    println!("logs {:?}", res.logs());

    header["number"] = json!("0x7d0");
    // use agg seal signed by validator 1 and 2, but ecdsa signed by validator 0
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f9016cd594908d0fdaeaefbb209bdcb540c2891e75616154b3f882b8802d692ebfd5b28f869cf87b12688504f1fd2194ad68d0bcbdde5f03ec45e98ef82e4114565378770ff9b81cc4488bbe93ba4dfaadf7a54c088560397588c1ab7c0de7dc40658ca64443100d757e9236555e7e1929edc3f398fa508ab5926bf1510eeabadcda0e475fcc4274349bdabdbf3c3855cc37548e2ebf7b0314436bee29f842b840285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e380b841bbc844fe92738c7ba8f2840a1cd95413a5630edf9d3b8985c6a7aff390e8b64f43e67a6640217a0af86cf1162f67d3e363e483e0afcff94d44dee170bcaed5d400f84403b84002af3de2d2a5ce5233a20b115839500a402939074446237a7336f7b00baab37011ac4324d2e763caa19e21b2b06daa6e6ee1a7f1df3e308f51f0e7db62be2c5f01c3808080");
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_12).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    assert!(res.err().unwrap().to_string().contains("the header's coinbase is not in validators"), "unexpected failure reason");

    Ok(())
}

#[tokio::test]
async fn test_validator_remove_02() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    // remove validator 0
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b840276d8fe5533b4d570fbe9bc0b104022f9c67f5210d493c9558aef517db0713e0255250efd11d89993aac000054bcebd3bea74665a163030555bfd0ad01d8bedf01c3808080");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header failed");
    println!("logs {:?}", res.logs());

    header["number"] = json!("0x7d0");
    header["coinbase"] = json!("0xEbf0E9FbC6210F199d1C34f2418b64129e7FF78A");
    // use agg seal signed by validator 1 and 2, and ecdsa signed by validator 1
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f9016cd594908d0fdaeaefbb209bdcb540c2891e75616154b3f882b8802d692ebfd5b28f869cf87b12688504f1fd2194ad68d0bcbdde5f03ec45e98ef82e4114565378770ff9b81cc4488bbe93ba4dfaadf7a54c088560397588c1ab7c0de7dc40658ca64443100d757e9236555e7e1929edc3f398fa508ab5926bf1510eeabadcda0e475fcc4274349bdabdbf3c3855cc37548e2ebf7b0314436bee29f842b840285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e380b841bbc844fe92738c7ba8f2840a1cd95413a5630edf9d3b8985c6a7aff390e8b64f43e67a6640217a0af86cf1162f67d3e363e483e0afcff94d44dee170bcaed5d400f84403b84002af3de2d2a5ce5233a20b115839500a402939074446237a7336f7b00baab37011ac4324d2e763caa19e21b2b06daa6e6ee1a7f1df3e308f51f0e7db62be2c5f01c3808080");
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_12).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header should succeed");


    Ok(())
}

#[tokio::test]
async fn test_validator_remove_add() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let mut header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    // remove validator 0
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b840276d8fe5533b4d570fbe9bc0b104022f9c67f5210d493c9558aef517db0713e0255250efd11d89993aac000054bcebd3bea74665a163030555bfd0ad01d8bedf01c3808080");
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header failed");
    println!("logs {:?}", res.logs());

    header["number"] = json!("0x7d0");
    header["coinbase"] = json!("0xEbf0E9FbC6210F199d1C34f2418b64129e7FF78A");
    // use agg seal signed by validator 1 and 2, ecdsa signed by validator 1, and add validator 0 back
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f9016cd594908d0fdaeaefbb209bdcb540c2891e75616154b3f882b8802d692ebfd5b28f869cf87b12688504f1fd2194ad68d0bcbdde5f03ec45e98ef82e4114565378770ff9b81cc4488bbe93ba4dfaadf7a54c088560397588c1ab7c0de7dc40658ca64443100d757e9236555e7e1929edc3f398fa508ab5926bf1510eeabadcda0e475fcc4274349bdabdbf3c3855cc37548e2ebf7b0314436bee29f842b840285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e380b841bbc844fe92738c7ba8f2840a1cd95413a5630edf9d3b8985c6a7aff390e8b64f43e67a6640217a0af86cf1162f67d3e363e483e0afcff94d44dee170bcaed5d400f84403b84002af3de2d2a5ce5233a20b115839500a402939074446237a7336f7b00baab37011ac4324d2e763caa19e21b2b06daa6e6ee1a7f1df3e308f51f0e7db62be2c5f01c3808080");
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_12).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header should succeed");
    println!("logs {:?}", res.logs());

    header["number"] = json!("0xbb8");
    // use agg seal signed by validator 1 and 2, and ecdsa signed by validator 1
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b8418f5825e9457a87e3a31ae7f25d46a5f9d6c33825738692dbc521dd47b36b2d8b2c80caad3e284a153d666a03005556a389379792fc9d99355aadbf49e73c3a2400f84403b8400f79b4ce695d451eddce75bd00fa75213256c8bfb1f4e1cf95e0905e4fbb31030ce249b6a22b361b36d004cdaf05d2d2d6774fdc2d8028f3ce63584d3c36bc3e01c3808080");
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_12).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "update_block_header should fail");
    println!("error: {}", res.as_ref().err().unwrap());
    assert!(res.err().unwrap().to_string().contains("threshold is not satisfied"), "get unexpected error");

    // use agg seal signed by validator 0, 1 and 2, and ecdsa signed by validator 0
    header["coinbase"] = json!("0x908D0FDaEAEFbb209BDcb540C2891e75616154b3");
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841903f61c9fa76fbd761e4a6ed2b86b4e9c1944f5ca7fb90cf91f40b8a1621adfd401a2147b83e32489d05449ad4f41bbcbd76a8fd987231f0ce2e4ebeb096dff500f8440bb8401f531bae5b23fcb998c11240dc657482614d3e3bf959c7d354d397bef86bbdd114ceee2042ed3e0c6d3d10d4d1af18bbfa1c6bb2ffb9c746372a91aecdc8a58701c3808080");
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header should succeed");

    Ok(())
}

#[tokio::test]
async fn test_get_header_height() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let height: U64 = contract
        .call(&worker, "get_header_height")
        .view()
        .await?
        .json()?;

    let epoch :u64 = init_args["epoch"].as_str().unwrap().parse().unwrap();
    let epoch_size: u64 = init_args["epoch_size"].as_str().unwrap().parse().unwrap();
    assert_eq!(epoch_size * (epoch - 1), height.0, "get_header_height get unexpected result");

    let header: serde_json::Value = serde_json::from_str(HEADER_0_012).unwrap();
    let agg_pk: serde_json::Value = serde_json::from_str(AGG_PK_012).unwrap();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!({
            "header": header,
            "agg_pk": agg_pk
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "update_block_header failed");

    let height: U64 = contract
        .call(&worker, "get_header_height")
        .args_json({})?
        .gas(300_000_000_000_000)
        .transact()
        .await?
        .json()?;

    let height_no_prefix = header["number"].as_str().unwrap().trim_start_matches("0x");
    let exp_height: u64 = u64::from_str_radix(height_no_prefix, 16).unwrap();
    assert_eq!(exp_height, height.0, "get_header_height get unexpected result");

    Ok(())
}

#[tokio::test]
async fn test_get_verifiable_header_range() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let range: (U64, U64) = contract
        .call(&worker, "get_verifiable_header_range")
        .view()
        .await?
        .json()?;
    assert_eq!(2001, range.0.0, "wrong min verifiable header");
    assert_eq!(3000, range.1.0, "wrong mac verifiable header");

    let file = fs::File::open("./tests/data/header.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mut block = 3000;
    while block <= 23000 {
        let value = headers[block.to_string()].clone();
        let res = contract
            .call(&worker, "update_block_header")
            .args_json(json!(value))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;

        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "update_block_header {} failed", block);

        block += 1000;
    }

    let range: (U64, U64) = contract
        .call(&worker, "get_verifiable_header_range")
        .view()
        .await?
        .json()?;
    assert_eq!(4001, range.0.0, "wrong min verifiable header");
    assert_eq!(24000, range.1.0, "wrong mac verifiable header");

    Ok(())
}

#[tokio::test]
async fn test_get_epoch_size() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let epoch_size: U64 = contract
        .call(&worker, "get_epoch_size")
        .view()
        .await?
        .json()?;

    let exp_epoch_size :u64 = init_args["epoch_size"].as_str().unwrap().parse().unwrap();
    assert_eq!(exp_epoch_size, epoch_size.0, "get_epoch_size get unexpected result");

    Ok(())
}

#[tokio::test]
async fn test_get_validators_for_epoch() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "new contract failed");

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "1"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_some(), "epoch 1 should have record");
    let record = record_opt.unwrap();
    let validators = &init_args["validators"];
    assert_eq!(3, record.threshold.0, "threshold check failed");
    assert_eq!(1, record.epoch.0, "epoch check failed");

    println!("validators:{:?}", validators);

    for (i, validator) in record.validators.iter().enumerate() {
        let val_exp: Validator = serde_json::from_str(validators[i].to_string().as_str()).unwrap();
        assert_eq!(val_exp, *validator, "validator check failed");
    }

    Ok(())
}

#[tokio::test]
async fn test_update_validator_for_20_epochs() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/header.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mut block = 3000;
    while block <= 23000 {
        let value = headers[block.to_string()].clone();
        let res = contract
            .call(&worker, "update_block_header")
            .args_json(json!(value))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;

        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "update_block_header {} failed", block);

        block += 1000;
    }

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "3"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_none(), "epoch 3 should have no record");

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "4"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_none(), "epoch 4 should have no record");

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "5"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_some(), "epoch 5 should have record");
    Ok(())
}

#[tokio::test]
async fn test_verify_proof() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!("187");
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["187133"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "verify_proof_data for block 187133 should fail");
    println!("error: {}", res.as_ref().err().unwrap());
    assert!(res.err().unwrap().to_string().contains("cannot get epoch record for block"),
            "should be epoch record not found error");

    let file = fs::File::open("./tests/data/header.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!( headers["187000"]))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 187000 failed");

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["187133"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data for block 187133 failed");

    Ok(())
}

#[tokio::test]
async fn test_add_validator() -> anyhow::Result<()> {
    let added_val = r#"{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":"1","address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"}"#;
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] =  json!("188");
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/header.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let header = headers["188000"].clone();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(header))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 188000 failed");

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "189"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_some(), "epoch 189 should have record");
    let record = record_opt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold.0, "threshold check failed");
    assert_eq!(189, record.epoch.0, "epoch check failed");
    assert_eq!(validators.len() + 1, record.validators.len(), "one validator should be added");

    for validator in record.validators.iter() {
        println!("{}", serde_json::to_string(validator).unwrap())
    }

    assert_eq!(added_val, serde_json::to_string(record.validators.last().unwrap()).unwrap());

    Ok(())
}

#[tokio::test]
async fn test_remove_validator() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let del_val = r#"{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":"1","address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"}"#;
    let validators = r#"[{"g1_pub_key":{"x":"0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1","y":"0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287"},"weight":"1","address":"0x053af2b1ccbacba47c659b977e93571c89c49654"},
{"g1_pub_key":{"x":"0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69","y":"0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7"},"weight":"1","address":"0xb47adf1e504601ff7682b68ba7990410b92cd958"},
{"g1_pub_key":{"x":"0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835","y":"0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042"},"weight":"1","address":"0xf655fc7c95c70a118f98b46ca5028746284349a5"},
{"g1_pub_key":{"x":"0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b","y":"0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37"},"weight":"1","address":"0xb243f68e8e3245464d21b79c7ceae347ecc08ea6"},
{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":"1","address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"}]
"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!("203");
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    init_args["threshold"] = json!("4");
    println!("validators:{}", init_args["validators"]);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/header.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let header = headers["203000"].clone();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(header))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 203000 failed");

    let record_opt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": "204"
        }))?
        .view()
        .await?
        .json()?;

    assert!(record_opt.is_some(), "epoch 204 should have record");
    let record = record_opt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(3, record.threshold.0, "threshold check failed");
    assert_eq!(204, record.epoch.0, "epoch check failed");
    assert_eq!(validators.len() - 1, record.validators.len(), "one validator should be removed");

    for validator in record.validators.iter() {
        println!("{}", serde_json::to_string(validator).unwrap());
        assert_ne!(del_val, serde_json::to_string(validator).unwrap());
    }

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_after_add_validator() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let validators = r#"[{"g1_pub_key":{"x":"0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1","y":"0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287"},"weight":"1","address":"0x053af2b1ccbacba47c659b977e93571c89c49654"},
{"g1_pub_key":{"x":"0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69","y":"0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7"},"weight":"1","address":"0xb47adf1e504601ff7682b68ba7990410b92cd958"},
{"g1_pub_key":{"x":"0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835","y":"0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042"},"weight":"1","address":"0xf655fc7c95c70a118f98b46ca5028746284349a5"},
{"g1_pub_key":{"x":"0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b","y":"0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37"},"weight":"1","address":"0xb243f68e8e3245464d21b79c7ceae347ecc08ea6"},
{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":"1","address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"}]
"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!("203");
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    println!("validators:{}", init_args["validators"]);
    init_args["threshold"] = json!("4");
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["202554"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data on block 202554 failed");

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_after_remove_validator() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!("206");
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["205002"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data on block 205002 failed");

    Ok(())
}

async fn deploy_contract() -> anyhow::Result<(Worker<Sandbox>, Contract)> {
    std::env::var(NEAR_SANDBOX_BIN_PATH).expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(&std::fs::read(MAP_CLIENT_WASM_FILEPATH)?)
        .await?;

    println!("deploy contract id: {:?}", contract.id());

    Ok((worker, contract))
}