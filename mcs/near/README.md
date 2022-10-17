
The project includes 4 types of contracts, which are:
1. **multisig contract**: owner account of map light client contract and mcs contract to avoid centralization risk
2. **mcs factory contract**: factory contract to create multisig contract and mcs contract
3. **mcs contract**: MAP cross chain service contract
4. **mcs token contract**: NEP-141 token created by mcs contract

# Pre-requisites

Make sure Rust development environment is installed and configured.

See [here](https://www.rust-lang.org/tools/install) about how to install.

# Build the contracts

**NOTE**:

If building RELEASE version, please set the parameter RELEASE to **true** in ./script/config.h.

If building TEST version, please set the parameter RELEASE to **false** in ./script/config.h.

Then run below command to build:

```shell
./scripts/build.sh
```
7 wasm files will be generated in directory ./script/res, which are: (the first 2 files are copied from mapclients project)
1. **map_client_factory.wasm**: factory contract to deploy and initialize the MAP light client contract and make the MAP light contract account in locked state.
2. **map_light_client.wasm**: MAP light client contract
3. **mcs.wasm**: MAP cross chain service contract
4. **mcs_factory.wasm**: factory contract to deploy and initialize the MCS contract and make MCS contract account in locked state.
5. **mcs_token.wasm**: NEP-141 token contract deployed by MCS contract
6. **mock_map_client.wasm**: mocked MAP light client contract which is for testing
7. **multisig.wasm**: multisig contract


# Deploy the contracts
1. deploy 2 factory contracts first
```shell
    RES_DIR=./script/res  # path to the res directory
    MASTER_ACCOUNT=xxx.near # make sure the account is already created on NEAR blockchain
    FACTORY_ACCOUNT0=yyy.$MASTER_ACCOUNT # the map client factory contract account to be created
    FACTORY_ACCOUNT1=zzz.$MASTER_ACCOUNT # the mcs factory contract account to be created
    
    echo "creating map client factory account"
    near create-account $FACTORY_ACCOUNT0 --masterAccount $MASTER_ACCOUNT --initialBalance 30
    
    echo "deploying map client factory contract"
    near deploy --accountId $FACTORY_ACCOUNT0 --wasmFile $RES_DIR/map_client_factory.wasm
    
    echo "creating mcs factory account"
    near create-account $FACTORY_ACCOUNT1 --masterAccount $MASTER_ACCOUNT --initialBalance 30
    
    echo "deploying mcs factory contract"
    near deploy --accountId $FACTORY_ACCOUNT1 --wasmFile $RES_DIR/mcs_factory.wasm
```
2. configure and deploy multisig contract 
```shell
    MULTISIG_NAME="multisig"  # multisig contract name, the account ID will be $MULTISIG_NAME.$FACTORY_ACCOUNT1
    MEMBER0=member0.$MASTER_ACCOUNT  # members to add and confirm the requests, should be created on NEAR blockchain already 
    MEMBER1=member1.$MASTER_ACCOUNT
    MEMBER2=member2.$MASTER_ACCOUNT
    NUM_CONFIRMS=2
    REQUEST_LOCK=600000000000  # time lock period in nanosecond
    
    INIT_ARGS_MULTISIG='{
                  "name":"'$MULTISIG_NAME'",
                  "members": [
                  {"account_id": "'$MEMBER0'"},
                  {"account_id": "'$MEMBER1'"},
                  {"account_id": "'$MEMBER2'"}
                  ],
                  "num_confirmations": '$NUM_CONFIRMS',
                  "request_lock": '$REQUEST_LOCK'
                }'
    
    echo "create and initialize multisig contract"
    near call $FACTORY_ACCOUNT1 create_multisig "$INIT_ARGS_MULTISIG" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 20
```

3. configure and deploy MAP light client contract
```shell
    CLIENT_NAME="client"   # MAP light client name, the account ID will be $CLIENT_NAME.$FACTORY_ACCOUNT0
    MULTISIG_ACCOUNT=$MULTISIG_NAME.$FACTORY_ACCOUNT1   # the multisig contract account ID which has already created
    THRESHOLD=3 # compute the threshold according to validators info
    EPOCH=300 # the next epoch of the block on which you get the snapshot
    EPOCH_SIZE=1000 # can be retrieved from the snapshot response of the MAP blockchain
    VALIDATORS='[
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
       ]' # can be retrieved from the snapshot response of the MAP blockchain
       
    INIT_ARGS_CLIENT='{
       "name": "'$CLIENT_NAME'",
       "owner": "'$MULTISIG_ACCOUNT'",
       "threshold": '$THRESHOLD',
       "epoch": '$EPOCH',
       "epoch_size": '$EPOCH_SIZE',
       "validators": '$VALIDATORS'
    }'
    
    echo "create and initialize map light client contract"
    near call $FACTORY_ACCOUNT0 create_map_client "$INIT_ARGS_CLIENT" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30
```

3. configure and deploy mcs contract
```shell

MCS_NAME="mcs"  # MCS contract name, the account ID will be $MCS_NAME.$FACTORY_ACCOUNT1
MULTISIG_ACCOUNT=$MULTISIG_NAME.$FACTORY_ACCOUNT1  # the multisig contract account ID which has already created
CLIENT_ACCOUNT=$CLIENT_NAME.$FACTORY_ACCOUNT0   # the MAP light client contract account ID which has already created
MAP_BRIDGE_ADDRESS="F579c89C22DAc92E9816C0b39856cA9281Bd7BE0"  # the MCS contract address on MAP blockchain
WNEAR_ACCOUNT="wrap.testnet"  # wrap NEAR account on NEAR blockchain, "wrap.near" for mainnet, "wrap.testnet" for testnet
NEAR_CHAIN_ID=1313161555 # NEAR chain ID, 1313161554 for mainnet, 1313161555 for testnet

INIT_ARGS_MCS='{
              "name":"'$MCS_NAME'",
              "owner": "'$MULTISIG_ACCOUNT'",
              "map_light_client": "'$CLIENT_ACCOUNT'",
              "map_bridge_address": "'$MAP_BRIDGE_ADDRESS'",
              "wrapped_token": "'$WNEAR_ACCOUNT'",
              "near_chain_id": '$NEAR_CHAIN_ID'
            }'

  echo "create and initialize mcs contract"
  near call $FACTORY_ACCOUNT1 create_mcs "$INIT_ARGS_MCS" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 30
```

# Usage

We can use the shell scripts in directory ./script to simplify the steps.

1. Support new NEP-141 mcs token to cross chain through MCS service
```shell
    SCRIPT_DIR=./script
    MCS_ACCOUNT=$MCS_NAME.$FACTORY_ACCOUNT1
    MCS_TOKEN_NAME="mcs_token_0"  # the mcs token name, the token account will be $MCS_TOKEN_NAME.$MCS_ACCOUNT
    MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT
    DECIMALS=24
    
    echo "deploying $MCS_TOKEN_NAME contract"
    near call $MCS_ACCOUNT deploy_mcs_token '{"address": "'$MCS_TOKEN_NAME'"}'  --accountId $MASTER_ACCOUNT --deposit 10 --gas 80000000000000
    
    echo "setting metadata by multisig contract"
    $SCRIPT_DIR/manage_multisig.sh request_and_confirm metadata $MCS_TOKEN $DECIMALS
    
    echo "listing mcs tokes"
    near view $MCS_ACCOUNT get_mcs_tokens '{}'
```

2. Add the target blockchain for mcs/ft/native token to transfer to
```shell
    SCRIPT_DIR=./script
    TO_CHAIN=212 # to chain ID
    CHAIN_TYPE="EvmChain"  # to chain type
    MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT  # mcs token account ID
    FT_TOKEN="wrap.testnet"  # ft token account ID
    
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
```

3. Transfer mcs/ft/native token to another blockchain through MCS service
```shell
    FROM="map001.testnet"
    TO="[46,120,72,116,221,179,44,215,151,93,104,86,91,80,148,18,165,181,25,244]"
    TO_CHAIN=212 # to chain ID
    MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT  # mcs token account ID
    FT_TOKEN="wrap.testnet"  # ft token account ID
    
    echo "transfer mcs token, make sure $FROM has enough $MCS_TOKEN"
    $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
    $SCRIPT_DIR/manage_mcs_token.sh transfer $MCS_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
    $SCRIPT_DIR/manage_mcs_token.sh balance $MCS_TOKEN $FROM
    
    echo "transfer ft token, make sure $FROM has enough $FT_TOKEN"
    $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
    $SCRIPT_DIR/manage_ft_token.sh transfer $FT_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
    $SCRIPT_DIR/manage_ft_token.sh balance $FT_TOKEN $FROM
    
    echo "transfer native token, make sure $FROM has enough NEAR"
    $SCRIPT_DIR/manage_native_token.sh balance $FROM
    $SCRIPT_DIR/manage_native_token.sh transfer $TO_CHAIN $FROM $TO $AMOUNT
    $SCRIPT_DIR/manage_native_token.sh balance $FROM
```

# Testing
1. How to run unit testing?

```shell
cargo test --workspace --lib
```

2. How to run integration testing?


**NOTE**: Before run the integration testing, make sure **near sandbox** exists on your computer.
If not, please clone the [nearcore](https://github.com/near/nearcore) project and run "make sandbox" to build it.


```shell
# set below environment before run tests
export NEAR_SANDBOX_BIN_PATH="/path/to/near/sandbox/bin"

cargo test
```