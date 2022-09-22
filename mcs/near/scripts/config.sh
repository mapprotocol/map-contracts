RELEASE=false  # set to true before building release version
MASTER_ACCOUNT="map003.testnet"
MAP_BRIDGE_ADDRESS="F579c89C22DAc92E9816C0b39856cA9281Bd7BE0"
WNEAR_ACCOUNT="wrap.testnet"
NEAR_CHAIN_ID=1313161555
MULTISIG_NAME="multisig"
MCS_NAME="mcs"
CLIENT_NAME="client"
REQUEST_LOCK=5000000000

FACTORY_ACCOUNT0=fac0.$MASTER_ACCOUNT
FACTORY_ACCOUNT1=fac1.$MASTER_ACCOUNT
MULTISIG_ACCOUNT=$MULTISIG_NAME.$FACTORY_ACCOUNT1
MCS_ACCOUNT=$MCS_NAME.$FACTORY_ACCOUNT1
CLIENT_ACCOUNT=$CLIENT_NAME.$FACTORY_ACCOUNT0
MEMBER0=member0.$MASTER_ACCOUNT
MEMBER1=member1.$MASTER_ACCOUNT
MEMBER2=member2.$MASTER_ACCOUNT

INIT_ARGS_MULTISIG='{
              "name":"'$MULTISIG_NAME'",
              "members": [
              {"account_id": "'$MEMBER0'"},
              {"account_id": "'$MEMBER1'"},
              {"account_id": "'$MEMBER2'"}
              ],
              "num_confirmations": 2,
              "request_lock": '$REQUEST_LOCK'
            }'

INIT_ARGS_MCS='{
              "name":"'$MCS_NAME'",
              "owner": "'$MULTISIG_ACCOUNT'",
              "map_light_client": "'$CLIENT_ACCOUNT'",
              "map_bridge_address": "'$MAP_BRIDGE_ADDRESS'",
              "wrapped_token": "'$WNEAR_ACCOUNT'",
              "near_chain_id": '$NEAR_CHAIN_ID'
            }'

INIT_ARGS_CLIENT='{
    "name": "'$CLIENT_NAME'",
    "owner": "'$MULTISIG_ACCOUNT'",
    "threshold": 3,
    "epoch": 300,
    "epoch_size": 1000,
    "validators": [
        {
            "g1_pub_key": {
                "x": "0x01370ecd3f4871a718079cb799ed57597b6087eb09811fae7635f541a0b14c57",
                "y": "0x1b327c6f9d07f6f2b666e341fa7cb3531ee510da50fedc567739a7040a1dc696"
            },
            "weight": 1,
            "address": "0xec3e016916ba9f10762e33e03e8556409d096fb4"
        },
        {
            "g1_pub_key": {
                "x": "0x2dc393cb4e1d6bb5e26c4fef0ccdde874535af1da42f64b34525a399dc1bbe62",
                "y": "0x1291bd0437dbb1f7ea7737ad515546b8f6b696ea0b9f6f49d5f6c039259ae778"
            },
            "weight": 1,
            "address": "0x6f08db5ba52d896f2472eb49580ac6d8d0351a66"
        },
        {
            "g1_pub_key": {
                "x": "0x2801781ffcf2371c911090b1dfe626a7b4e745810f30d545e45b965674bee6b3",
                "y": "0x23ef4f51b21bd4d141e484ff8f9d5becddc4ffe0d432a80d59b982aab1f9e575"
            },
            "weight": 1,
            "address": "0x2f3079a1c1c0995a1c9803853d1b8444cce0aa9f"
        },
        {
            "g1_pub_key": {
                "x": "0x1d330a79f1374d37c618bcb34edc38f99935a9f44d3885672232495e22fce151",
                "y": "0x2b742d040ff3e9a996b79406cc4f18fc6c9b4a28ee7c3e88590406259f404531"
            },
            "weight": 1,
            "address": "0x096bf1097f3af73b716eab545001d97b2cf1fb20"
        }
    ]
}'