set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  prepare                                 create member accounts"
  echo "  clean                                   delete member account"
  echo "  request_and_confirm <request type> <member> add request and confirm by member"
  echo "  request_type:"
  echo "    add_native <chain id>                    add native token to_chain"
  echo "    add_mcs    <token> <chain id>            add mcs token to_chain"
  echo "    add_ft    <token> <chain id>             add fungible token to_chain"
  echo "    remove_native <chain id>                 remove native token to_chain"
  echo "    remove_mcs    <token> <chain id>         remove mcs token to_chain"
  echo "    remove_ft    <token> <chain id>          remove fungible token to_chain"
  echo "    upgrade_mcs  <wasm file>                 upgrade mcs contract"
  echo "    upgrade_mcs_token <token>  <wasm file>   upgrade mcs token contract"
  echo "    set_client  <map client account>         set new map light client account to mcs contract"
  echo "    set_owner  <multisig account>            set new multisig light client account to mcs contract"
  echo "    set_paused  <mask>                       set paused flag to mcs contract"
  echo "  confirm <request id> <member>              confirm request"
  echo "  execute <request id> <account>             execute confirmed request"
  echo "  help                                       show help"
}

function prepare() {
  near create-account $MEMBER0 --masterAccount $MASTER_ACCOUNT --initialBalance 1
  near create-account $MEMBER1 --masterAccount $MASTER_ACCOUNT --initialBalance 1
  near create-account $MEMBER2 --masterAccount $MASTER_ACCOUNT --initialBalance 1
}

function prepare_request() {
  case $1 in
    add_native)
      if [[ $# == 3 ]]; then
        echo "adding native token to_chain $2 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_native_to_chain"
        ARGS=`echo '{"to_chain": '$2'}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    add_mcs)
      if [[ $# == 4 ]]; then
        echo "add mcs token $2 to_chain $3 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_mcs_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    add_ft)
      if [[ $# == 4 ]]; then
        echo "add ft token $2 to_chain $3 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_fungible_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    remove_native)
      if [[ $# == 3 ]]; then
        echo "remove native token to_chain $2 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_native_to_chain"
        ARGS=`echo '{"to_chain": '$2'}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    remove_mcs)
      if [[ $# == 3 ]]; then
        echo "remove mcs token $2 to_chain $3 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_mcs_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    remove_ft)
      if [[ $# == 4 ]]; then
        echo "remove ft token $2 to_chain $3 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_fungible_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    metadata)
      if [[ $# == 4 ]]; then
        echo "set metadata of mcs token $2's decimals to $3"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_metadata"
        ARGS=`echo '{"address": "'$2'", "decimals": '$3'}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    chain_type)
      if [[ $# == 4 ]]; then
        echo "set chain type of chain $2 to $3"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_chain_type"
        ARGS=`echo '{"chain_id": '$2', "chain_type": "'$3'"}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    upgrade_mcs)
      if [[ $# == 3 ]]; then
        echo "upgrade mcs contract to $2"
        RECEIVER=$MCS_ACCOUNT
        METHOD="upgrade_self"
        CODE=`base64 -i $2`
        ARGS=`echo '{"code": "'$CODE'"}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    upgrade_mcs_token)
      if [[ $# == 4 ]]; then
        echo "upgrade mcs token $2 to $3"
        RECEIVER=$2
        METHOD="upgrade_self"
        CODE=`base64 -i $3`
        ARGS=`echo '{"code": "'$CODE'"}'| base64`
        MEMBER=$4
      else
        printHelp
        exit 1
      fi
      ;;
    set_client)
      if [[ $# == 3 ]]; then
        echo "set map light client account of mcs contract to $2"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_map_light_client"
        ARGS=`echo '{"map_client_account": "'$2'"}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    set_owner)
      if [[ $# == 3 ]]; then
        echo "set multisig owner of mcs contract to $2"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_owner"
        ARGS=`echo '{"new_owner": "'$2'"}'| base64`
        MEMBER=$3
      else
        printHelp
        exit 1
      fi
      ;;
    set_paused)
      if [[ $# == 3 ]]; then
        echo "set mcs contract paused flag to $2"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_paused"
        ARGS=`echo '{"paused": '$2'}'| base64`
        MEMBER=$3
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

}

function confirm() {
  echo "confirming request id '$1' by member '$2'"
  near view $MULTISIG_ACCOUNT get_request '{"request_id": '$1'}'
  near view $MULTISIG_ACCOUNT get_confirmations '{"request_id": '$1'}'
  near call $MULTISIG_ACCOUNT confirm '{"request_id": '$1'}' --accountId $2 --gas 300000000000000
}

function execute() {
  echo "executing request id '$1' by account '$2'"
  near view $MULTISIG_ACCOUNT get_request '{"request_id": '$1'}'
  near view $MULTISIG_ACCOUNT get_confirmations '{"request_id": '$1'}'
  near call $MULTISIG_ACCOUNT execute '{"request_id": '$1'}' --accountId $2 --gas 300000000000000
}

function request_and_confirm() {
  prepare_request $@

  near call $MULTISIG_ACCOUNT add_request_and_confirm '{
    "request": {
      "receiver_id": "'$RECEIVER'",
      "actions": [
        {
          "type": "FunctionCall",
            "method_name": "'$METHOD'",
            "args": "'$ARGS'",
            "deposit": "0",
            "gas": "150000000000000"
        }
      ]
    }
  }' --accountId $MEMBER --gas 300000000000000
}

function clean() {
    near delete $MEMBER0 $MASTER_ACCOUNT
    near delete $MEMBER1 $MASTER_ACCOUNT
    near delete $MEMBER2 $MASTER_ACCOUNT
}


if [[ $# -gt 0 ]]; then
  case $1 in
    prepare)
      prepare
      ;;
    clean)
      clean
      ;;
    request)
      shift
      request $@
      ;;
    request_and_confirm)
      shift
      request_and_confirm $@
      ;;
    confirm)
      if [[ $# == 3 ]]; then
        shift
        confirm $@
      else
        printHelp
        exit 1
      fi
      ;;
    execute)
      if [[ $# == 3 ]]; then
        shift
        execute $@
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
