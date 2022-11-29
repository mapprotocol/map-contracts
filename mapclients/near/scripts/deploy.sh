set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

RESPONSE=`curl -X POST $MAP_RPC_URL -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"istanbul_getEpochInfo","params":['$EPOCH_ID'],"id":1}'`
INIT_ARGS_CLIENT=`echo $RESPONSE | jq  .result | jq --arg name $CLIENT_NAME '.name=$name'`
INIT_ARGS_CLIENT=`echo $INIT_ARGS_CLIENT | jq --args '.owner = "'$OWNER'"'`

FACTORY_ACCOUNT=$FACTORY_NAME.$MASTER_ACCOUNT
echo $MASTER_ACCOUNT
echo $FACTORY_ACCOUNT
echo $INIT_ARGS_CLIENT

echo "creating map client factory account"
near create-account $FACTORY_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30

echo "deploying map light client factory contract"
near deploy --accountId $FACTORY_ACCOUNT --wasmFile $RES_DIR/map_client_factory.wasm

echo "create and initialize map light client contract"
near call $FACTORY_ACCOUNT create_map_client "$INIT_ARGS_CLIENT" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30
