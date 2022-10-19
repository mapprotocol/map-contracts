set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

REQUEST_LOCK_IN_NS=$((REQUEST_LOCK*1000000000))
INIT_ARGS_MULTISIG='{
              "name":"'$MULTISIG_NAME'",
              "members": [
              ],
              "num_confirmations": '$CONFIRMS',
              "request_lock": '$REQUEST_LOCK_IN_NS'
            }'
for MEMBER in "${MEMBERS[@]}"
do
    INIT_ARGS_MULTISIG=`echo $INIT_ARGS_MULTISIG | jq --args '.members += [{"account_id": "'$MEMBER'"}]'`
done

echo $INIT_ARGS_MULTISIG

INIT_ARGS_MCS='{
              "name":"'$MCS_NAME'",
              "owner": "'$MULTISIG_ACCOUNT'",
              "map_light_client": "'$CLIENT_ACCOUNT'",
              "map_bridge_address": "'$MAP_MCS_ADDRESS'",
              "wrapped_token": "'$WNEAR_ACCOUNT'",
              "near_chain_id": '$NEAR_CHAIN_ID'
            }'

echo $INIT_ARGS_MCS

echo "creating mcs factory account"
near create-account $MCS_FACTORY_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30

echo "deploying mcs factory contract"
near deploy --accountId $MCS_FACTORY_ACCOUNT --wasmFile $RES_DIR/mcs_factory.wasm

echo "creating multisig contract"
near call $MCS_FACTORY_ACCOUNT create_multisig "$INIT_ARGS_MULTISIG" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 20

echo "creating mcs contract"
near call $MCS_FACTORY_ACCOUNT create_mcs "$INIT_ARGS_MCS" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30