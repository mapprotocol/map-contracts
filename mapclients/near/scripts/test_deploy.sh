set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

MASTER_ACCOUNT=map007.testnet # make sure the account is already created on NEAR blockchain
CLIENT_ACCOUNT=client.map007.testnet # the name of MAP light client contract to be created, the account ID will be $CLIENT_NAME.$FACTORY_NAME
MAP_RPC_URL=https://testnet-rpc.maplabs.io  # the RPC url of MAP blockchain
EPOCH_ID=157  # get the information of this epoch id to initialize the MAP light client contract
OWNER=map007.testnet # the owner of the MAP light client, which is a multisig-timelock contract

RESPONSE=`curl -X POST $MAP_RPC_URL -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"istanbul_getEpochInfo","params":['$EPOCH_ID'],"id":1}'`
INIT_ARGS_CLIENT=`echo $RESPONSE | jq  .result `
INIT_ARGS_CLIENT=`echo $INIT_ARGS_CLIENT | jq --args '.owner = "'$OWNER'"'`

#FACTORY_ACCOUNT=$FACTORY_NAME.$MASTER_ACCOUNT
echo $MASTER_ACCOUNT
echo $CLIENT_ACCOUNT
echo $INIT_ARGS_CLIENT

echo "creating map client factory account"
near create-account $CLIENT_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 30

echo "deploying map light client factory contract"
near deploy --accountId $CLIENT_ACCOUNT --wasmFile $RES_DIR/map_light_client.wasm

echo "create and initialize map light client contract"
near call $CLIENT_ACCOUNT new "$INIT_ARGS_CLIENT" --accountId $MASTER_ACCOUNT --gas 300000000000000
