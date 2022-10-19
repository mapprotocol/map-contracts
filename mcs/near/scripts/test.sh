set -e

SCRIPT_DIR=$(dirname $0)
source $SCRIPT_DIR/config.sh
FILE_NAME=$0

MCS_TOKEN_NAME="mcs_token_0"
MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT
DECIMALS=24
FT_TOKEN="wrap.testnet"
AMOUNT=100000000000000000000000

FROM="pandarr.testnet"
TO="[46,120,72,116,221,179,44,215,151,93,104,86,91,80,148,18,165,181,25,244]"
#TO_CHAIN=34434
TO_CHAIN=22776 # map
CHAIN_TYPE="EvmChain"

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  init                           init members, factory, map client, multisig and mcs contracts"
  echo "  deinit                         clean members and factory contracts"
  echo "  prepare                        prepare tokens and set to chain"
  echo "  t  <token type>                transfer out token"
  echo "  d  <token type>                deposit out token"
  echo "  add_chain                      add to chain for tokens"
  echo "  clean                          delete mcs token, mcs contract and map client contract"
  echo "  help                           show help"
  echo '  <token type> could be "mcs", "ft" or "native"'
}

function transfer_mcs() {
  $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
  $SCRIPT_DIR/manage_mcs_token.sh transfer $MCS_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
}

function transfer_ft() {
  $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
  $SCRIPT_DIR/manage_ft_token.sh transfer $FT_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
}

function transfer_native() {
  $SCRIPT_DIR/manage_native_token.sh balance $FROM
  $SCRIPT_DIR/manage_native_token.sh transfer $TO_CHAIN $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_native_token.sh balance $FROM
}

function deposit_mcs() {
  $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
  $SCRIPT_DIR/manage_mcs_token.sh deposit $MCS_TOKEN $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
}

function deposit_ft() {
  $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
  $SCRIPT_DIR/manage_ft_token.sh deposit $FT_TOKEN $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
}

function deposit_native() {
  $SCRIPT_DIR/manage_native_token.sh balance $FROM
  $SCRIPT_DIR/manage_native_token.sh deposit $FROM $TO $AMOUNT
  $SCRIPT_DIR/manage_native_token.sh balance $FROM
}

function init() {
  echo "preparing multisig members"
  $SCRIPT_DIR/manage_multisig.sh prepare

  echo "preparing factory contract"
  $SCRIPT_DIR/manage_factory.sh deploy
  echo "creating map light client, multisig and mcs contract"
  $SCRIPT_DIR/manage_factory.sh create
}

function deinit() {
  echo "clean multisig members"
  $SCRIPT_DIR/manage_multisig.sh clean

  echo "clean factory contracts"
  $SCRIPT_DIR/manage_factory.sh clean
}

function add_chain() {
  echo "setting chain type"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm chain_type $TO_CHAIN $CHAIN_TYPE

  echo "adding mcs token to_chain"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm add_mcs $MCS_TOKEN $TO_CHAIN
  $SCRIPT_DIR/manage_mcs_token.sh list

  echo "adding ft token to_chain"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm add_ft $FT_TOKEN $TO_CHAIN
  $SCRIPT_DIR/manage_ft_token.sh list

  echo "adding native token to_chain"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm add_native $TO_CHAIN
  $SCRIPT_DIR/manage_native_token.sh list
}

function prepare() {
  echo "preparing mcs token"
  $SCRIPT_DIR/manage_mcs_token.sh deploy $MCS_TOKEN_NAME
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm metadata $MCS_TOKEN $DECIMALS
  $SCRIPT_DIR/manage_mcs_token.sh list

  echo "minting 100000000000000000000000 $MCS_TOKEN for account $FROM"
  near call $MCS_TOKEN mint '{"account_id": "'$FROM'", "amount": "100000000000000000000000000"}' --accountId $MASTER_ACCOUNT --deposit 0.01

  add_chain
}

function clean() {
  echo "upgrading mcs token $MCS_TOKEN to mock map client"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm upgrade_mcs_token $MCS_TOKEN $SCRIPT_DIR/res/mock_map_client.wasm
  echo "deleting mcs toke $MCS_TOKEN"
  near call $MCS_TOKEN delete_self '{"beneficiary":"'$MASTER_ACCOUNT'"}' --accountId $MASTER_ACCOUNT

  echo "upgrading mcs contract $MCS_ACCOUNT to mock map client"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm upgrade_mcs $SCRIPT_DIR/res/mock_map_client.wasm
  echo "deleting mcs contract $MCS_ACCOUNT"
  near call $MCS_ACCOUNT delete_self '{"beneficiary":"'$MASTER_ACCOUNT'"}' --accountId $MASTER_ACCOUNT

  echo "upgrading map light client contract $CLIENT_ACCOUNT to mock map client"
  $SCRIPT_DIR/manage_multisig.sh request_and_confirm upgrade_client $SCRIPT_DIR/res/mock_map_client.wasm
  echo "deleting map client contract $CLIENT_ACCOUNT"
  near call $CLIENT_ACCOUNT delete_self '{"beneficiary":"'$MASTER_ACCOUNT'"}' --accountId $MASTER_ACCOUNT
}


if [[ $# -gt 0 ]]; then
  case $1 in
    init)
      init
      ;;
    deinit)
      deinit
      ;;
    prepare)
      prepare
      ;;
    add_chain)
      add_chain
      ;;
    clean)
      clean
      ;;
    t)
      if [[ $# == 2 ]]; then
        case $2 in
        mcs)
          transfer_mcs
          ;;
        ft)
          transfer_ft
          ;;
        native)
          transfer_native
          ;;
        *)
          printHelp
          exit 1
          ;;
        esac
      else
        printHelp
        exit 1
      fi
      ;;
    d)
      if [[ $# == 2 ]]; then
        case $2 in
        mcs)
          deposit_mcs
          ;;
        ft)
          deposit_ft
          ;;
        native)
          deposit_native
          ;;
        *)
          printHelp
          exit 1
          ;;
        esac
      else
        printHelp
        exit 1
      fi
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
