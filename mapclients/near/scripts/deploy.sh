set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh
INIT_ARGS_CLIENT=`cat $SCRIPT_DIR/init_args_client.json`

echo $MASTER_ACCOUNT
echo $CLIENT_ACCOUNT
echo $INIT_ARGS_CLIENT

near create-account $CLIENT_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30

echo "deploying map light client contract"
near deploy --accountId $CLIENT_ACCOUNT --wasmFile $RES_DIR/map_light_client.wasm

echo "initializing map light client  contract"
near call $CLIENT_ACCOUNT new "$INIT_ARGS_CLIENT" --accountId $MASTER_ACCOUNT --gas 80000000000000
