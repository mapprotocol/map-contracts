MASTER_ACCOUNT="XXX.testnet"
INIT_ARGS_MCS='{
              "map_light_client": "client1.'$MASTER_ACCOUNT'",
              "map_bridge_address": "603E956f28549F4791D83CE6d9c80b0C271CD864",
              "wrapped_token": "wrap.testnet",
              "near_chain_id": 1313161555
            }'
MCS_ACCOUNT=mcs1.$MASTER_ACCOUNT

near create-account $MCS_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 40

echo "deploying mcs contract"
near deploy --accountId $MCS_ACCOUNT --wasmFile ../target/wasm32-unknown-unknown/release/mcs.wasm

echo "initializing mcs contract"
near call $MCS_ACCOUNT init "$INIT_ARGS_MCS" --accountId $MASTER_ACCOUNT --gas 80000000000000

echo "deploying mcs_token_0 contract"
near call $MCS_ACCOUNT deploy_mcs_token '{"address": "mcs_token_0"}'  --accountId $MASTER_ACCOUNT --deposit 10 --gas 80000000000000

echo "getting mcs_token list from mcs contract"
near view $MCS_ACCOUNT get_mcs_tokens '{}'

