set -e

SCRIPT_DIR=$(dirname $0)
source $SCRIPT_DIR/config.sh
FILE_NAME=$0

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  list                                              view registered fungible tokens and their to chains"
  echo "  transfer <token> <to chain> <from> <to> <amount>  transfer out ft token"
  echo "  deposit <token> <from> <to> <amount>              deposit out ft token"
  echo "  swap <token> <from> <to chain> <to> <amount>              swap out ft token"
  echo "  balance <token> <account>                         view account balance of ft token"
  echo "  help                                              show help"
}

function list_tokens() {
  echo "getting fungible token list from mcs contract"
  near view $MCS_ACCOUNT get_fungible_tokens '{}'
}

function transfer_out() {
  echo "transfer out $5 $1 token from $3 to $4 on chain $2"
  near call $1 ft_transfer_call '{"receiver_id":"'$MCS_ACCOUNT'", "amount":"'$5'", "memo": "", "msg": "{\"msg_type\": 0, \"to\": '$4', \"to_chain\": \"'$2'\"}"}' --accountId $3 --depositYocto 1 --gas 60000000000000
}

function deposit_out() {
  echo "deposit out $4 $1 token from $2 to $3 on MAP chain"
  near call $1 ft_transfer_call '{"receiver_id":"'$MCS_ACCOUNT'", "amount":"'$4'", "memo": "", "msg": "{\"msg_type\": 1, \"to\": '$3', \"to_chain\": \"0\"}"}' --accountId $2 --depositYocto 1 --gas 60000000000000
}

function swap_out() {
  echo "swap out $5 $1 token from $3 to $4 on chain $2"
  near call $1 ft_transfer_call '{"receiver_id":"'$MCS_ACCOUNT'", "amount":"'$5'", "memo": "", "msg": "{ \"to\": '$4', \"to_chain\": \"'$2'\", \"swap_info\": {\"src_swap\":[{\"amount_in\":\"1000\",\"min_amount_out\":\"1\",\"path\":\"token1.map007.testnetXtoken2.map007.testnet\",\"router_index\":\"1\"}],"dst_swap":{\"swap_param\":[],\"target_token\":[116,111,107,101,110,50,46,109,97,112,48,48,55,46,116,101,115,116,110,101,116],\"map_target_token\":\"0x0101010101010101010101010101010101010101\"}}}"}' --accountId $3 --depositYocto 1 --gas 60000000000000
}

function balance() {
  echo "get account $2 balance of token $1"
  near view $1 ft_balance_of '{"account_id":"'$2'"}'
}

if [[ $# -gt 0 ]]; then
  case $1 in
    list)
      if [[ $# == 1 ]]; then
        list_tokens
      else
        printHelp
        exit 1
      fi
      ;;
    transfer)
      if [[ $# == 6 ]]; then
        shift
        transfer_out $@
      else
        printHelp
        exit 1
      fi
      ;;
    deposit)
      if [[ $# == 5 ]]; then
        shift
        deposit_out $@
      else
        printHelp
        exit 1
      fi
      ;;
    balance)
      if [[ $# == 3 ]]; then
        shift
        balance $@
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
