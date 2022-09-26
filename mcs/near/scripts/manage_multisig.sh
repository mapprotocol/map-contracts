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
  echo "  request_and_confirm <request type> <args> add request and confirm by member0"
  echo "  request_type:"
  echo "    add_native <chain id>                    add native token to_chain"
  echo "    add_mcs    <token> <chain id>            add mcs token to_chain"
  echo "    add_ft    <token> <chain id>             add fungible token to_chain"
  echo "    remove_native <chain id>                 remove native token to_chain"
  echo "    remove_mcs    <token> <chain id>         remove mcs token to_chain"
  echo "    remove_ft    <token> <chain id>          remove fungible token to_chain"
  echo "    upgrade_mcs  <wasm file>                 upgrade mcs contract"
  echo "    upgrade_mcs_token <token>  <wasm file>   upgrade mcs token contract"
  echo "    upgrade_client  <wasm file>              upgrade map light client contract"
  echo "  confirm <request id> <member>              confirm request"
  echo "  execute <request id>                       execute confirmed request"
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
      if [[ $# == 2 ]]; then
        echo "adding native token to_chain $2 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_native_to_chain"
        ARGS=`echo '{"to_chain": '$2'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    add_mcs)
      if [[ $# == 3 ]]; then
        echo "add mcs token $2 to_chain $3 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_mcs_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    add_ft)
      if [[ $# == 3 ]]; then
        echo "add ft token $2 to_chain $3 to mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="add_fungible_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    remove_native)
      if [[ $# == 2 ]]; then
        echo "remove native token to_chain $2 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_native_to_chain"
        ARGS=`echo '{"to_chain": '$2'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    remove_mcs)
      if [[ $# == 2 ]]; then
        echo "remove mcs token $2 to_chain $3 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_mcs_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    remove_ft)
      if [[ $# == 3 ]]; then
        echo "remove ft token $2 to_chain $3 from mcs contract"
        RECEIVER=$MCS_ACCOUNT
        METHOD="remove_fungible_token_to_chain"
        ARGS=`echo '{"token": "'$2'", "to_chain": '$3'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    metadata)
      if [[ $# == 3 ]]; then
        echo "set metadata of mcs token $2's decimals to $3"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_metadata"
        ARGS=`echo '{"address": "'$2'", "decimals": '$3'}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    chain_type)
      if [[ $# == 3 ]]; then
        echo "set chain type of chain $2 to $3"
        RECEIVER=$MCS_ACCOUNT
        METHOD="set_chain_type"
        ARGS=`echo '{"chain_id": '$2', "chain_type": "'$3'"}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    upgrade_mcs)
      if [[ $# == 2 ]]; then
        echo "upgrade mcs contract to $2"
        RECEIVER=$MCS_ACCOUNT
        METHOD="upgrade_self"
        CODE=`base64 -i $2`
        ARGS=`echo '{"code": "'$CODE'"}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    upgrade_mcs_token)
      if [[ $# == 3 ]]; then
        echo "upgrade mcs token $2 to $3"
        RECEIVER=$2
        METHOD="upgrade_self"
        CODE=`base64 -i $3`
        ARGS=`echo '{"code": "'$CODE'"}'| base64`
      else
        printHelp
        exit 1
      fi
      ;;
    upgrade_client)
      if [[ $# == 2 ]]; then
        echo "upgrade map client contract to $2"
        RECEIVER=$CLIENT_ACCOUNT
        METHOD="upgrade"
        CODE=`base64 -i $2`
        ARGS=`echo '{"code": "'$CODE'"}'| base64`
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
  echo "executing request id '$1'"
  near view $MULTISIG_ACCOUNT get_request '{"request_id": '$1'}'
  near view $MULTISIG_ACCOUNT get_confirmations '{"request_id": '$1'}'
  near call $MULTISIG_ACCOUNT execute '{"request_id": '$1'}' --accountId $MASTER_ACCOUNT --gas 300000000000000
}

function request_and_confirm() {
  TMP_LOG=/tmp/output.log

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
  }' --accountId $MEMBER0 --gas 300000000000000 > $TMP_LOG

  cat $TMP_LOG
  ID=`cat $TMP_LOG|tail -n 1`

  DELAY_TIME=$((REQUEST_LOCK/1000000000 + 10))
  echo "delay $DELAY_TIME before next conform"
  sleep $DELAY_TIME
  confirm $ID $MEMBER1 > $TMP_LOG

  cat $TMP_LOG
  KEY_WORDS="cannot be executed before time"
  while [[ `cat $TMP_LOG` =~ $KEY_WORDS ]]; do
    sleep 5
    execute $ID > $TMP_LOG
    cat $TMP_LOG
    KEY_WORDS="smaller than min execute time"
  done

  echo "finish request and confirm"
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
      if [[ $# == 2 ]]; then
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
