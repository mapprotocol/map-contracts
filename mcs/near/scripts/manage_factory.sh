set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  deploy                              deploy and init factory contract"
  echo "  redeploy                            redeploy factory contract"
  echo "  clean                               delete factory contract account"
  echo "  create                              deploy and init map light client, multisig and mcs contract"
  echo "  help                                show help"
}

function deploy() {
  near create-account $FACTORY_ACCOUNT0 --masterAccount $MASTER_ACCOUNT --initialBalance 30

  echo "deploying map client factory contract"
  near deploy --accountId $FACTORY_ACCOUNT0 --wasmFile $RES_DIR/map_client_factory.wasm

  near create-account $FACTORY_ACCOUNT1 --masterAccount $MASTER_ACCOUNT --initialBalance 30

  echo "deploying mcs factory contract"
  near deploy --accountId $FACTORY_ACCOUNT1 --wasmFile $RES_DIR/mcs_factory.wasm
}

function redeploy() {
  echo "redeploying map client factory contract"
  near deploy --accountId $FACTORY_ACCOUNT0 --wasmFile $RES_DIR/map_client_factory.wasm

  echo "redeploying mcs factory contract"
  near deploy --accountId $FACTORY_ACCOUNT1 --wasmFile $RES_DIR/mcs_factory.wasm
}

function clean() {
    near delete $FACTORY_ACCOUNT0 $MASTER_ACCOUNT
    near delete $FACTORY_ACCOUNT1 $MASTER_ACCOUNT
}

function create() {
  echo "creating map light client contract"
  near call $FACTORY_ACCOUNT0 create_map_client "$INIT_ARGS_CLIENT" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30

  echo "creating multisig contract"
  near call $FACTORY_ACCOUNT1 create_multisig "$INIT_ARGS_MULTISIG" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 20

  echo "creating mcs contract"
  near call $FACTORY_ACCOUNT1 create_mcs "$INIT_ARGS_MCS" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30
}


if [[ $# -eq 1 ]]; then
  case $1 in
    deploy)
      deploy
      ;;
    redeploy)
      redeploy
      ;;
    clean)
      clean
      ;;
    create)
      create
      ;;
    help)
      printHelp
      ;;
    *)
      echo "Unknown command $1"
      printHelp
      exit 1
      ;;
  esac
  else
    printHelp
    exit 1
fi
