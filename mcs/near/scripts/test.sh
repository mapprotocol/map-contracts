set -e

SCRIPT_DIR=$(dirname $0)
source $SCRIPT_DIR/config.sh
FILE_NAME=$0

MCS_TOKEN_NAME="mcs_token_0"
MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT
DECIMALS=24
FT_TOKEN="wrap.testnet"
AMOUNT=1000000000000000000000

FROM="pandarr.testnet"
TO="[207,200,11,237,219,112,241,42,246,218,118,143,195,14,57,104,137,223,206,38]"
TO_CHAIN=34434

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  prepare                        prepare tokens"
  echo "  t  <token type>                transfer out token"
  echo "  d  <token type>                deposit out token"
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

function prepare() {
  echo "preparing mcs token"
  $SCRIPT_DIR/manage_mcs_token.sh deploy $MCS_TOKEN_NAME
  $SCRIPT_DIR/manage_mcs_token.sh metadata $MCS_TOKEN $DECIMALS
  $SCRIPT_DIR/manage_mcs_token.sh add $MCS_TOKEN $TO_CHAIN
  $SCRIPT_DIR/manage_mcs_token.sh list

  echo "preparing ft token"
  $SCRIPT_DIR/manage_ft_token.sh add $FT_TOKEN $TO_CHAIN
  $SCRIPT_DIR/manage_ft_token.sh list

  echo "preparing native token"
  $SCRIPT_DIR/manage_native_token.sh add $TO_CHAIN
  $SCRIPT_DIR/manage_native_token.sh list

  echo "minting 100000000000000000000000 $MCS_TOKEN for account $FROM"
  near call $MCS_TOKEN mint '{"account_id": "'$FROM'", "amount": "100000000000000000000000"}' --accountId $MCS_ACCOUNT
}


if [[ $# -gt 0 ]]; then
  case $1 in
    prepare)
      prepare
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
