set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  deploy                              deploy and init mcs contract"
  echo "  redeploy                            redeploy mcs contract"
  echo "  clean                               delete mcs contract account"
  echo "  help                                show help"
}

function deploy() {
  near create-account $MCS_ACCOUNT --masterAccount $MASTER_ACCOUNT --initialBalance 40

  echo "deploying mcs contract"
  near deploy --accountId $MCS_ACCOUNT --wasmFile $RES_DIR/mcs.wasm

  echo "initializing mcs contract"
  near call $MCS_ACCOUNT init "$INIT_ARGS_MCS" --accountId $MASTER_ACCOUNT --gas 80000000000000
}

function redeploy() {
  echo "redeploying mcs contract"
  near deploy --accountId $MCS_ACCOUNT --wasmFile $RES_DIR/mcs.wasm
}

function clean() {
    near delete $MCS_ACCOUNT $MASTER_ACCOUNT
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
