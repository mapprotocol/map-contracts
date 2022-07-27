MASTER_ACCOUNT="XXX.testnet"
INIT_ARGS_CLIENT=`cat data/init_args_client.json`
INIT_ARGS_MCS='{
              "map_light_client": "client.'$MASTER_ACCOUNT'",
              "map_bridge_address": "e2123fa0c94db1e5baeff348c0e7aecd15a11b45",
              "wrapped_token": "wrap.testnet",
              "near_chain_id": 1313161555
            }'

# near create-account client.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30
# near create-account mcs.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30

near deploy --accountId client.$MASTER_ACCOUNT --wasmFile ~/WorkMap/map-contracts/mapclients/near/target/wasm32-unknown-unknown/release/map_light_client.wasm

near call client.$MASTER_ACCOUNT new INIT_ARGS_CLIENT --accountId $MASTER_ACCOUNT

echo "deploying mcs contract"
near deploy --accountId mcs.$MASTER_ACCOUNT --wasmFile ~/WorkMap/map-contracts/mcs/near/target/wasm32-unknown-unknown/release/mcs.wasm

echo "initializing mcs contract"
near call mcs.$MASTER_ACCOUNT init $INIT_ARGS_MCS --accountId $MASTER_ACCOUNT

# deploy 2 mcs token contracts
echo "deploying mcs_token_0 contract"
near call mcs.$MASTER_ACCOUNT deploy_mcs_token '{"address": "mcs_token_0"}'  --accountId $MASTER_ACCOUNT --deposit 10 --gas 60000000000000
echo "deploying mcs_token_1 contract"
near call mcs.$MASTER_ACCOUNT deploy_mcs_token '{"address": "mcs_token_1"}'  --accountId $MASTER_ACCOUNT --deposit 10 --gas 60000000000000

echo "getting mcs_token list from mcs contract"
near view mcs.$MASTER_ACCOUNT get_mcs_tokens '{}'

echo "adding mcs_token_0 to_chain info to mcs contract"
near call mcs.$MASTER_ACCOUNT add_mcs_token_to_chain '{"token": "mcs_token_0", "to_chain": 1}' --accountId mcs.$MASTER_ACCOUNT

echo "minting 1000 mcs token for "$MASTER_ACCOUNT
near call mcs.$MASTER_ACCOUNT mint '{"token": "mcs_token_0", "to":"'$MASTER_ACCOUNT'", "amount": 1000}' --accountId mcs.$MASTER_ACCOUNT
near view mcs_token_0.mcs.$MASTER_ACCOUNT ft_balance_of '{"account_id":"'$MASTER_ACCOUNT'"}'

echo "transfer out 100 mcs token from "$MASTER_ACCOUNT" to 0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4 on chain 1"
near call mcs.$MASTER_ACCOUNT transfer_out_token '{"token":"mcs_token_0", "to":"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4", "amount": 100, "to_chain": 1}' --accountId $MASTER_ACCOUNT --deposit 5 --gas 60000000000000

echo "deploying ft contract"
near deploy --accountId ft.$MASTER_ACCOUNT --wasmFile ~/WorkMap/map-contracts/mcs/near/target/wasm32-unknown-unknown/release/mcs_token.wasm
echo "initializing ft contract"
near call ft.$MASTER_ACCOUNT new '{}' --accountId ft.$MASTER_ACCOUNT
echo "adding ft token to_chain info to mcs contract"
near call mcs.$MASTER_ACCOUNT add_fungible_token_to_chain '{"token": "ft.'$MASTER_ACCOUNT'", "to_chain": 1}' --accountId mcs.$MASTER_ACCOUNT

echo "minting 1000 ft token for "$MASTER_ACCOUNT
near call ft.$MASTER_ACCOUNT mint '{"account_id":"'$MASTER_ACCOUNT'", "amount": "1000"}' --accountId ft.$MASTER_ACCOUNT --deposit 5
near view ft.$MASTER_ACCOUNT ft_balance_of '{"account_id":"'$MASTER_ACCOUNT'"}'

echo "registering storage for account mcs."$MASTER_ACCOUNT
near call ft.$MASTER_ACCOUNT storage_deposit '{"account_id":"mcs.'$MASTER_ACCOUNT'", "registration_only": true}' --accountId $MASTER_ACCOUNT --deposit 1
echo "transfer out 100 mcs token from "$MASTER_ACCOUNT" to 0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4 on chain 1"
near call ft.$MASTER_ACCOUNT ft_transfer_call '{"receiver_id":"mcs.'$MASTER_ACCOUNT'", "amount":"100", "memo": "", "msg": "{\"typ\": 0, \"to\": \"0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4\", \"to_chain\": 1}"}' --accountId $MASTER_ACCOUNT --depositYocto 1 --gas 60000000000000
