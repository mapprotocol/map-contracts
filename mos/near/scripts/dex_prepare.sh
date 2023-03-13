set -e

SCRIPT_DIR=$(dirname $0)
source $SCRIPT_DIR/config.sh
FILE_NAME=$0

REF_DEX=ref-finance-101.testnet

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  deploy-token <name> <decimal> <master account>        deploy NEP-141 token"
  echo "  mint <token> <user> <amount>                          mint token for user"
  echo "  create-pool <token0> <token1>                         create pool on ref exchange"
  echo "  view-pool <pool id>                                   view pool on ref exchange"
  echo "  add-liquidity <pool id> <amount0> <amount1>           add liquidity to pool"
  echo "  help                                                  show help"
}


function deploy_token() {
  token=$1.$3
  echo "creating account $token"
  near create-account $token --masterAccount $3 --initialBalance 5
  echo "deploying token contract to $token"
  near deploy $token --wasmFile=$SCRIPT_DIR/res/mcs_token.wasm
  echo "initializing token contract $token"
  near call $token new '{"owner": "'$3'"}' --accountId $3
  near call $token set_metadata '{"address": "'$token'", "decimals": '$2'}' --accountId $3
}

function mint() {
  owner=`echo $1 | cut -d . -f 2-5`
  echo "owner $owner will mint $3 $1 for user $2"
  near call $1 mint '{"account_id": "'$2'", "amount":"'$3'"}' --accountId $owner --deposit 1
}

function create_pool() {
    owner=`echo $1 | cut -d . -f 2-5`
    echo $1 $2
    near call $REF_DEX register_tokens "{\"token_ids\": [\"$1\", \"$2\"]}" --accountId $owner --depositYocto 1
    near call $REF_DEX add_simple_pool "{\"tokens\": [\"$1\", \"$2\"], \"fee\": 25}" --accountId $owner --amount 0.1
}

function view_pool() {
    near view $REF_DEX get_pool '{"pool_id": '$1'}'
}

function add_liquidity() {
   tokens=`near view $REF_DEX get_pool '{"pool_id": '$1'}' | grep token_account_ids`
   token0=`echo $tokens | awk -F "'" '{print $2}'`
   token1=`echo $tokens | awk -F "'" '{print $4}'`

    echo "pool $1 has token $token0 and $token1"
    owner=`echo $token0 | cut -d . -f 2-5`

    near call $token0 storage_deposit '{"account_id": "'$REF_DEX'"}' --accountId $owner --amount 0.0125
    near call $token0 ft_transfer_call "{\"receiver_id\": \"$REF_DEX\", \"amount\": \"$2\", \"msg\": \"\"}" --accountId $owner --amount 0.000000000000000000000001 --gas 300000000000000

    near call $token1 storage_deposit '{"account_id": "'$REF_DEX'"}' --accountId $owner --amount 0.0125
    near call $token1 ft_transfer_call "{\"receiver_id\": \"$REF_DEX\", \"amount\": \"$3\", \"msg\": \"\"}" --accountId $owner --amount 0.000000000000000000000001 --gas 300000000000000

    near call $REF_DEX add_liquidity '{"pool_id": '$1', "amounts": ["'$2'", "'$3'"]}' --accountId $owner --deposit 1
}


if [[ $# -gt 0 ]]; then
  case $1 in
    deploy-token)
      if [[ $# == 4 ]]; then
        shift
        deploy_token $@
      else
        printHelp
        exit 1
      fi
      ;;
    mint)
      if [[ $# == 4 ]]; then
        shift
        mint $@
      else
        printHelp
        exit 1
      fi
      ;;
    create-pool)
      if [[ $# == 3 ]]; then
        shift
        create_pool $@
      else
        printHelp
        exit 1
      fi
      ;;
    view-pool)
      if [[ $# == 2 ]]; then
        shift
        view_pool $@
      else
        printHelp
        exit 1
      fi
      ;;
    add-liquidity)
      if [[ $# == 4 ]]; then
        shift
        add_liquidity $@
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
