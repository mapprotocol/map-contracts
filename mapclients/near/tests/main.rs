use std::fs;
use std::ops::Index;
// macro allowing us to convert human readable units to workspace units.
// use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::{prelude::*, Worker, Contract};
use workspaces::network::{Sandbox, Testnet};
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
            "threshold": 3,
            "validators":
                [
                    {
                        "g1_pub_key":{"x":"0x285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae","y":"0x218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e3"},
                        "address":"0x908D0FDaEAEFbb209BDcb540C2891e75616154b3",
                        "weight": 1
                    },
                    {
                        "g1_pub_key":{"x":"0x0d570979e84f504247c0ab6c1bc98967a300192132707a6d144cce74d77ab11a","y":"0x28feb22d09573a136a1ae43f0329f77be54968035d7b29161de64068b52fa0fb"},
                        "address":"0xEbf0E9FbC6210F199d1C34f2418b64129e7FF78A",
                        "weight": 1
                    },
                    {
                        "g1_pub_key":{"x":"0x06123bea2fdc5ca96f7b3810d7abb489bd04fa3db73487e82261bb0a768d9686","y":"0x1958a18770f574432dd56665f8214ee885780c00de9e47f8a20e7b7075fa2448"},
                        "address":"0x8f189338912AC69AB776318A32Ad7473731a955F",
                        "weight": 1
                    },
                    {
                        "g1_pub_key":{"x":"0x055c69baeedb58db6e1467eb7d1d51347ffe8bc9e30be2cf8638d8bf9b9b9a53","y":"0x1c13d30bd973eabbc38c87b8ca3a846db126fcf62bfebe8516ca0b26f959b9ff"},
                        "address":"0xD762eD84dB64848366E74Ce43742C1960Ba62304",
                        "weight": 1
                    }
                ],
            "epoch":1,
            "epoch_size":1000
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
                "extra":"0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841abb150fcf44735d6df641c0b3f29f3a6b5088542031734e70ab48beec303aaff4e0a56372af7774be1df2240916de4b329dd747b38b53e258ff79af6ac93f3d901f84407b840262c29d794b767971c4bad1d409a12f1aefeea93c1a08a6d50585e90f6497c5105810086251448d011d7010b4c97f82756ff7c0d3e4b82f59b725971972c685d01c3808080",
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

    let init_args: serde_json::Value = serde_json::from_str(INIT_VALUE).unwrap();
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
async fn test_update_block_header_38000() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(38);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(headers["38000"]))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 38000 failed");

    Ok(())
}

#[tokio::test]
async fn test_update_block_header_93000() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(93);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(headers["93000"]))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 93000 failed");

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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841b166f38dd2c80eaf82c185ae6fbfb3c4ad34eb24e784e2277f9427540b9d066804868983df05e962c72cc43e3f865a0d4e36eeef08ac4a5dcea2d286ba79540e01f84407b84006772298021df72315132405e180c495c7d010e4ed5cf633de743c38fd4a17b21b7f1a608e9b71925a8daae486eb103292a00a4e12f344b2312974dc773b644101c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841abb150fcf44735d6df641c0b3f29f3a6b5088542031734e70ab48beec303aaff4e0a56372af7774be1df2240916de4b329dd747b38b53e258ff79af6ac93f3d901f84403b8401a55b4a158281a57b587072ef196e0482c445a2b0942ad1349b1b221416a53be1c0eb2b4c284c49c8ebed1bb3eb52a3bc56bab4d3da12ecd6860c7920eb9c20e01c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b84151bcd0f46fa9ec5d0d8ba37741e6336b7bf3c4de7077121f86f36c007690a5fe21db3dc88866bfd4a7455242f7ad7bf5e8484c5376d7495aac7b7c07b8ebfe2f00f84407b8401a32291646f4bc327b2d75cd670c5c523653a3aa70c82501d6c95feb4c9000e322791113b87d7a09043516e40cf22abeb20066e57918e32ab7d62973f47c375001c3808080");
    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({
            "receipt_proof" : {
            "header": header,
            "agg_pk": agg_pk,
            "receipt": {
                    "receipt_type": 1,
                    "post_state_or_status": "0x00",
                    "cumulative_gas_used": 2000,
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
                    "receipt_type": 1,
                    "post_state_or_status": "0x01",
                    "cumulative_gas_used": 2000,
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b84028e45d0dc6167e5db2aed90517e162eae66bb68ca701b15a17173f92eeaadda810320ba7313ad99cdefb70e90347a7b55472f9cca8fe47b663d218bc01433ef701c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b8417865a77a9d1cbe9ed0e38806cb06c469514f4f2cc9fecd38e7e5cb7787b8694534a65cb0a9377e5935131acbc34a94076e0a746f9f4887983e244b573cfdff4401f84406b84020d8e519b3df674ac821de4a8de5a3e27a0b0a6c4ef17a3f302c5ed98c3c2eef28d02b4f4c01beaeecbb061d91f21cb4851c0ee77727dbb51e509521b9f1ae5d01c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b84028e45d0dc6167e5db2aed90517e162eae66bb68ca701b15a17173f92eeaadda810320ba7313ad99cdefb70e90347a7b55472f9cca8fe47b663d218bc01433ef701c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b8415bc867a42c6c2db1b8a5e0d809903be9eb3d6b8435cb7d5cb373aa4755ba6b5a42c0ba3ec97b3b527eb6fa7659dc3ee05ba0b3f2ecd976aa51ea048b1fc480b000f84403b840121fc9c9d9dc5742b0d4b6fa26007a232e4f5ca407035741d8c964745bf9dd4a29fa3f4a508ee9eafa8543a473b61650ff5a8d98b020d63513daa1e7df4b5a6001c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c001b841b17d45ba282f76645e4af7b1653174cc3adb24fd9d1119f4238eb5f93c5647ed4728858861b0e96dacba7759df205c3f7a862cd345da975296b75cb419485f9600f84407b84028e45d0dc6167e5db2aed90517e162eae66bb68ca701b15a17173f92eeaadda810320ba7313ad99cdefb70e90347a7b55472f9cca8fe47b663d218bc01433ef701c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f9016cd594908d0fdaeaefbb209bdcb540c2891e75616154b3f882b8802d692ebfd5b28f869cf87b12688504f1fd2194ad68d0bcbdde5f03ec45e98ef82e4114565378770ff9b81cc4488bbe93ba4dfaadf7a54c088560397588c1ab7c0de7dc40658ca64443100d757e9236555e7e1929edc3f398fa508ab5926bf1510eeabadcda0e475fcc4274349bdabdbf3c3855cc37548e2ebf7b0314436bee29f842b840285b454a87ab802bca118adb5d36ec205e0aa2f373afc03555d91e41cbfffbae218a5545ea930860c0b99462596ee86f3278a5207c42bd63cb8dfaa54e0d68e380b841bbc844fe92738c7ba8f2840a1cd95413a5630edf9d3b8985c6a7aff390e8b64f43e67a6640217a0af86cf1162f67d3e363e483e0afcff94d44dee170bcaed5d400f84403b8400fac2e71b35a6c1e4a4ef8e9c5bfc9816f910e69d736db21da38fcc798b6dfbf1abd32c9ea55d0c33a52179d6e202201beb3b3e0556b797ec11970a82227149301c3808080");
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
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b8418f5825e9457a87e3a31ae7f25d46a5f9d6c33825738692dbc521dd47b36b2d8b2c80caad3e284a153d666a03005556a389379792fc9d99355aadbf49e73c3a2400f84406b8401f646b2bc9fe13cb7da308b06eb45b56ee277113b0803cb44e96b48f21eede8f199d6137fee9b5493a34c3acae3add7da337039b1a2d438bc17023943d6a325401c3808080");
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
    assert!(res.err().unwrap().to_string().contains("threshold is not satisfied"), "get unexpected error");

    // use agg seal signed by validator 0, 1 and 2, and ecdsa signed by validator 0
    header["coinbase"] = json!("0x908D0FDaEAEFbb209BDcb540C2891e75616154b3");
    header["extra"] = json!("0x0000000000000000000000000000000000000000000000000000000000000000f891c0c0c080b841903f61c9fa76fbd761e4a6ed2b86b4e9c1944f5ca7fb90cf91f40b8a1621adfd401a2147b83e32489d05449ad4f41bbcbd76a8fd987231f0ce2e4ebeb096dff500f8440bb8402f1dc01d96f11bd515e6003a61cca36f86be9fef5f39ed71f488c9b526e89bab1a2c7fea45ee55cf39388c46e12996f4f3891346634f90843377e8137e8993e001c3808080");
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

    let height: u64 = contract
        .call(&worker, "get_header_height")
        .view()
        .await?
        .json()?;

    let epoch = init_args["epoch"].as_u64().unwrap();
    let epoch_size = init_args["epoch_size"].as_u64().unwrap();
    assert_eq!(epoch_size * (epoch - 1), height, "get_header_height get unexpected result");

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

    let height: u64 = contract
        .call(&worker, "get_header_height")
        .args_json({})?
        .gas(300_000_000_000_000)
        .transact()
        .await?
        .json()?;

    let height_no_prefix = header["number"].as_str().unwrap().trim_start_matches("0x");
    let exp_height: u64 = u64::from_str_radix(height_no_prefix, 16).unwrap();
    assert_eq!(exp_height, height, "get_header_height get unexpected result");

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

    let epoch_size: u64 = contract
        .call(&worker, "get_epoch_size")
        .view()
        .await?
        .json()?;

    let exp_epoch_size = init_args["epoch_size"].as_u64().unwrap();
    assert_eq!(exp_epoch_size, epoch_size, "get_epoch_size get unexpected result");

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

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 1
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 1 should have record");
    let record = recordOpt.unwrap();
    let validators = &init_args["validators"];
    assert_eq!(3, record.threshold, "threshold check failed");
    assert_eq!(1, record.epoch, "epoch check failed");

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

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mut block = 1000;
    while block <= 20000 {
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

    Ok(())
}

#[tokio::test]
async fn test_update_validator_for_21_epochs() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let validators = r#"[{"g1_pub_key":{"x":"0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f249","y":"0x2b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3"},"weight":1,"address":"0xb4e1bc0856f70a55764fd6b3f8dd27f2162108e9"},
{"g1_pub_key":{"x":"0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf58","y":"0x1ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b"},"weight":1,"address":"0x7a3a26123dbd9cfefc1725fe7779580b987251cb"},
{"g1_pub_key":{"x":"0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c812","y":"0x1e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869"},"weight":1,"address":"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4"},
{"g1_pub_key":{"x":"0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea6","y":"0x0dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"},"weight":1,"address":"0x65b3fee569bf82ff148bdded9c3793fb685f9333"},
{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":1,"address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"},
{"g1_pub_key":{"x":"0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d","y":"0x2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"},"weight":1,"address":"0x4ca1a81e4c46b90ec52371c063d5721df61e7e12"}]
"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    init_args["epoch"] = json!(180);
    init_args["threshold"] = json!(4);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mut block = 180000;
    while block <= 200000 {
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

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 180
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_none(), "epoch 180 should have no record");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 181
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_none(), "epoch 181 should have  no record");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 182
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 182 should have record");

    let record = recordOpt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold, "threshold check failed");
    assert_eq!(182, record.epoch, "epoch check failed");
    assert_eq!(validators.len(), record.validators.len(), "no validator should be added/removed");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 201
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 201 should have record");

    let record = recordOpt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold, "threshold check failed");
    assert_eq!(201, record.epoch, "epoch check failed");
    assert_eq!(validators.len() - 1, record.validators.len(), "one validator should be removed");

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_in_diff_epochs() -> anyhow::Result<()> {
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

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let mut block = 1000;
    while block <= 3000 {
        let value = headers[block.to_string()].clone();
        let res = contract
            .call(&worker, "update_block_header")
            .args_json(json!( value))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;

        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "update_block_header {} failed", block);

        block += 1000;
    }

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["2568"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data for block 2568 failed");

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["4108"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await;

    assert!(res.is_err(), "verify_proof_data for block 4108 should fail");

    while block <= 5000 {
        let value = headers[block.to_string()].clone();
        let res = contract
            .call(&worker, "update_block_header")
            .args_json(json!( value))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;

        println!("logs {:?}", res.logs());
        assert!(res.is_success(), "update_block_header {} failed", block);

        block += 1000;
    }

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["4108"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data for block 4108 should success");

    Ok(())
}

#[tokio::test]
async fn test_add_validator() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(124);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let header = headers["124000"].clone();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(header))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 124000 failed");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 125
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 125 should have record");
    let record = recordOpt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold, "threshold check failed");
    assert_eq!(125, record.epoch, "epoch check failed");
    assert_eq!(validators.len() + 1, record.validators.len(), "one validator should be added");


    let header = headers["125000"].clone();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(header))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 125000 failed");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 126
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 126 should have record");
    let record = recordOpt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold, "threshold check failed");
    assert_eq!(126, record.epoch, "epoch check failed");
    assert_eq!(validators.len() + 2, record.validators.len(), "one validator should be added");

    for validator in record.validators.iter() {
        println!("{}", serde_json::to_string(validator).unwrap())
    }

    Ok(())
}

#[tokio::test]
async fn test_remove_validator() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let validators = r#"[{"g1_pub_key":{"x":"0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f249","y":"0x2b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3"},"weight":1,"address":"0xb4e1bc0856f70a55764fd6b3f8dd27f2162108e9"},
{"g1_pub_key":{"x":"0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf58","y":"0x1ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b"},"weight":1,"address":"0x7a3a26123dbd9cfefc1725fe7779580b987251cb"},
{"g1_pub_key":{"x":"0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c812","y":"0x1e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869"},"weight":1,"address":"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4"},
{"g1_pub_key":{"x":"0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea6","y":"0x0dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"},"weight":1,"address":"0x65b3fee569bf82ff148bdded9c3793fb685f9333"},
{"g1_pub_key":{"x":"0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a","y":"0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116"},"weight":1,"address":"0x98efa292822eb7b3045c491e8ae4e82b3b1ac005"},
{"g1_pub_key":{"x":"0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d","y":"0x2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"},"weight":1,"address":"0x4ca1a81e4c46b90ec52371c063d5721df61e7e12"}]
"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(190);
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    init_args["threshold"] = json!(4);
    println!("validators:{}", init_args["validators"]);
    let res = contract
        .call(&worker, "new")
        .args_json(json!(init_args))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(res.is_success(), "init contract failed!");

    let file = fs::File::open("./tests/data/headers.json").unwrap();
    let headers: serde_json::Value = serde_json::from_reader(file).unwrap();

    let header = headers["190000"].clone();
    let res = contract
        .call(&worker, "update_block_header")
        .args_json(json!(header))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "update_block_header 190000 failed");

    let recordOpt: Option<EpochRecord> = contract
        .call(&worker, "get_record_for_epoch")
        .args_json(json!({
            "epoch": 191
        }))?
        .view()
        .await?
        .json()?;

    assert!(recordOpt.is_some(), "epoch 191 should have record");
    let record = recordOpt.unwrap();
    let validators = &init_args["validators"].as_array().unwrap();
    assert_eq!(4, record.threshold, "threshold check failed");
    assert_eq!(191, record.epoch, "epoch check failed");
    assert_eq!(validators.len() - 1, record.validators.len(), "one validator should be removed");

    for validator in record.validators.iter() {
        println!("{}", serde_json::to_string(validator).unwrap())
    }

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_after_add_remove_validators_001() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

    let validators = r#"[
    {"g1_pub_key":{"x":"0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f249","y":"0x2b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3"},"weight":1,"address":"0xb4e1bc0856f70a55764fd6b3f8dd27f2162108e9"},
    {"g1_pub_key":{"x":"0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf58","y":"0x1ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b"},"weight":1,"address":"0x7a3a26123dbd9cfefc1725fe7779580b987251cb"},
    {"g1_pub_key":{"x":"0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c812","y":"0x1e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869"},"weight":1,"address":"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4"},
    {"g1_pub_key":{"x":"0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea6","y":"0x0dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"},"weight":1,"address":"0x65b3fee569bf82ff148bdded9c3793fb685f9333"},
    {"g1_pub_key":{"x":"0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d","y":"0x2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"},"weight":1,"address":"0x4ca1a81e4c46b90ec52371c063d5721df61e7e12"}
]"#;
    let file = fs::File::open("./tests/data/init_value.json").unwrap();
    let mut init_args: serde_json::Value = serde_json::from_reader(file).unwrap();
    init_args["epoch"] = json!(203);
    init_args["validators"] = serde_json::from_str(validators).unwrap();
    println!("validators:{}", init_args["validators"]);
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
        .args_json(json!({"receipt_proof": proofs["202351"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data on block 202351 failed");

    Ok(())
}

#[tokio::test]
async fn test_verify_proof_after_add_remove_validators_002() -> anyhow::Result<()> {
    let (worker, contract) = deploy_contract().await?;

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

    let file = fs::File::open("./tests/data/proof.json").unwrap();
    let proofs: serde_json::Value = serde_json::from_reader(file).unwrap();

    let res = contract
        .call(&worker, "verify_proof_data")
        .args_json(json!({"receipt_proof": proofs["449308"]}))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("logs {:?}", res.logs());
    assert!(res.is_success(), "verify_proof_data on block 449308 failed");

    Ok(())
}

async fn prepare_data() -> anyhow::Result<()> {
    let file = fs::File::open("./tests/data/updateHeader.json").unwrap();
    let mut json: serde_json::Value = serde_json::from_reader(file).unwrap();
    println!("json value: {}", json["1000"]);

    let mut i = 1000;
    while i <= 20000 {
        let block = i.to_string();
        json[&block]["header"]["number"] = json!(format!("0x{:x}", i));
        let time_str = json[&block]["header"]["time"].as_str().unwrap();
        let time: u64 = time_str[1..].parse().unwrap();
        json[&block]["header"]["time"] = json!(format!("0x{:x}", time));

        println!("number: {}, time: {}", &json[&block]["header"]["number"], &json[&block]["header"]["time"]);
    }

    let file = fs::File::create("./tests/data/headers.json").unwrap();
    serde_json::to_writer(file, &json).unwrap();

    Ok(())
}

async fn deploy_contract() -> anyhow::Result<(Worker<Testnet>, Contract)> {
    // std::env::set_var(NEAR_SANDBOX_BIN_PATH, "/Users/rong/Projects/near/nearcore/target/debug/neard-sandbox");
    // std::env::var(NEAR_SANDBOX_BIN_PATH).expect("environment variable NEAR_SANDBOX_BIN_PATH should be set");

    let worker = workspaces::testnet().await?;
    let contract = worker
        .dev_deploy(&std::fs::read(MAP_CLIENT_WASM_FILEPATH)?)
        .await?;

    println!("deploy contract id: {:?}", contract.id());

    Ok((worker, contract))
}