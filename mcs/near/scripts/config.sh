MASTER_ACCOUNT="map001.testnet"
MCS_ACCOUNT=mcs.$MASTER_ACCOUNT
CLIENT_ACCOUNT=client.$MASTER_ACCOUNT
MAP_BRIDGE_ADDRESS="F579c89C22DAc92E9816C0b39856cA9281Bd7BE0"
WNEAR_ACCOUNT="wrap.testnet"
NEAR_CHAIN_ID=1313161555
INIT_ARGS_MCS='{
              "map_light_client": "'$CLIENT_ACCOUNT'",
              "map_bridge_address": "'$MAP_BRIDGE_ADDRESS'",
              "wrapped_token": "'$WNEAR_ACCOUNT'",
              "near_chain_id": '$NEAR_CHAIN_ID'
            }'

