# MAP cross-chain service

The project includes 4 types of contracts, which are:
1. **multisig contract**: owner account of map light client contract and mcs contract to avoid centralization risk
2. **mcs factory contract**: factory contract to create multisig contract and mcs contract
3. **mcs contract**: MAP cross chain service contract
4. **mcs token contract**: NEP-141 token created by mcs contract

## Pre-requisites

**1. rust**

Follow [these instructions](https://doc.rust-lang.org/book/ch01-01-installation.html) for setting up Rust.
Then, add the **wasm32-unknown-unknown** toolchain which enables compiling Rust to Web Assembly (wasm), the low-level language used by the NEAR platform.

```shell
# Get Rust in linux and MacOS
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Add the wasm toolchain
rustup target add wasm32-unknown-unknown
```

**2. near-cli**
   
The NEAR Command Line Interface (CLI) is a tool that enables to interact with the NEAR network directly from the shell.
Follow [here](https://docs.near.org/tools/near-cli) for installing near-cli. 
Then, select the network and login with your master account.

```shell
# Install near-cli in linux and McsOS
npm install -g near-cli

# The default network for near-cli is testnet, change the network by setting NEAR_ENV
# export NEAR_ENV=mainnet

# login with your master account
near login
```

**3. jq**

Jq is a lightweight and flexible command-line JSON processor. Follow [here](https://stedolan.github.io/jq/download/) to install it.

## Build the contracts

Run below script to build:

```shell
./scripts/build.sh
```

5 wasm files will be generated in directory ./script/res, which are: (the first 2 files are copied from mapclients project)
1. **mcs.wasm**: MAP cross chain service contract
2. **mcs_factory.wasm**: factory contract to deploy and initialize the MCS contract and make MCS contract account in locked state.
3. **mcs_token.wasm**: NEP-141 token contract deployed by MCS contract
4. **mock_map_client.wasm**: mocked MAP light client contract which is for testing
5. **multisig.wasm**: multisig contract


## Deploy the contracts
**1. Configure below parameters in ./scripts/config.sh**

```shell
MASTER_ACCOUNT="map002.testnet" # make sure the account is already created on NEAR blockchain

# factory contract
FACTORY_NAME=mfac # the name of mcs factory contract to be created, the account ID will be $MFACTORY_NAME.$MASTER_ACCOUNT

# multisig contract
MULTISIG_NAME="multisig" # the name of multisig contract to be created, the account ID will be $MULTISIG_NAME.$MFACTORY_NAME.$MASTER_ACCOUNT
MEMBERS=(member0.map002.testnet member1.map002.testnet member2.map002.testnet)  # the multisig members list, make sure 
                                                                                # these accounts have been created on NEAR blockchain
CONFIRMS=2  # the multisig confirmation number to trigger the execution of the request
REQUEST_LOCK=5 # request cooldown period in seconds (time before a request can be executed)

# mcs contract
MCS_NAME="mcs"  # the name of mcs contract to be created, the account ID will be $MCS_NAME.$MFACTORY_NAME.$MASTER_ACCOUNT
MAP_MCS_ADDRESS="F579c89C22DAc92E9816C0b39856cA9281Bd7BE0"  # the mcs contract address on MAP relay chain
WNEAR_ACCOUNT="wrap.testnet"  # wrapped near contract account on NEAR blockchain
NEAR_CHAIN_ID=5566818579631833089  # NEAR testnet blockchain id, mainnet is 5566818579631833088
MAP_CHAIN_ID=22776  # MAP blockchain ID
CLIENT_ACCOUNT="client.fac.map002.testnet" # the account ID of the map light client contract which has already been deployed
```

**2. Deploy factory contract, multisig contract and mcs contract, and initialize them with below command:**
```shell
    ./scripts/deploy.sh
```


## Usage

We can use the shell scripts in directory ./script to simplify the steps. First run below command to set environment variables:

```shell
source ./scripts/config.sh
```

**NOTE**: in the following examples we are using 2 out of 3 multisig schema.


**1. Support new NEP-141 mcs token to cross chain through MCS service**
```shell
    MCS_TOKEN_NAME="mcs_token_0"  # the mcs token name, the token account will be $MCS_TOKEN_NAME.$MCS_ACCOUNT
    MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT # mcs token account ID
    DECIMALS=24
    USER_ACCOUNT="map002.testnet"  # the account to deploy new mcs token contract, make sure it is created on NEAR blockchain
    
    # deploy mcs token contract
    ./scripts/manage_mcs_token.sh deploy $MCS_TOKEN_NAME $USER_ACCOUNT
    
    # request to set metadata by multisig member
    ./scripts/manage_multisig.sh request_and_confirm metadata $MCS_TOKEN $DECIMALS ${MEMBERS[1]}
    
    # the request ID can be obtained from the last line of last command's output
    REQUEST_ID=
    
    # confirm the request by another member
    ./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}
    
    # if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
    # ./scripts/manage_multisig.sh execute $REQUEST_ID $USER_ACCOUNT
    
    # list mcs tokes
    ./scripts/manage_mcs_token.sh list
```

**2. Allow the mcs/ft/native token to transfer to a specified target blockchain**

First, we should set the chain type of target blockchain. Currently only **EvmChain** type is supported.
```shell
    TO_CHAIN=212 # to chain ID
    CHAIN_TYPE="EvmChain"  # to chain type
    
    # request to set chain type by multisig member
    ./scripts/manage_multisig.sh request_and_confirm chain_type $TO_CHAIN $CHAIN_TYPE ${MEMBERS[1]}
    
    # the request ID can be obtained from the last line of last command's output
    REQUEST_ID=
    
    # confirm the request by another member
    ./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}
    
    # if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
    # ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

Then, we should register the token to MOS.
```shell
    TOKEN="usdt.map007.testnet" # token Account Id
    Mintable=true               # the token is mintable
    
    # register the token
    ./scripts/manage_ft_token.sh register $TOKEN $Mintable
```

If you want to add target chain ID to mcs token, run below commands:

```shell
    TO_CHAIN=212 # to chain ID
    MCS_TOKEN_NAME="mcs_token_0"
    MCS_TOKEN=$MCS_TOKEN_NAME.$MCS_ACCOUNT  # mcs token account ID
    
    # request to add target chain ID to mcs token by multisig member
    ./scripts/manage_multisig.sh request_and_confirm add_mcs $MCS_TOKEN $TO_CHAIN ${MEMBERS[1]}
    
    # the request ID can be obtained from the last line of last command's output
    REQUEST_ID=
    
    # confirm the request by another member
    ./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}
    
    # if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
    # ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
    
    # view the token list to check if the chain ID is added successfully
    ./scripts/manage_mcs_token.sh list
```

If you want to add target chain ID to ft token, run below commands:

```shell
    TO_CHAIN=212 # to chain ID
    FT_TOKEN="wrap.testnet"  # ft token account ID
    
    # request to add target chain ID to ft token by multisig member
    ./scripts/manage_multisig.sh request_and_confirm add_ft $FT_TOKEN $TO_CHAIN ${MEMBERS[1]}
    
    # the request ID can be obtained from the last line of last command's output
    REQUEST_ID=
    
    # confirm the request by another member
    ./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}
    
    # if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
    # ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
    
    # view the token list to check if the chain ID is added successfully
    ./scripts/manage_ft_token.sh list
```

If you want to add target chain ID to native token, run below commands:

```shell
    TO_CHAIN=212 # to chain ID
    
    # request to add target chain ID to native token by multisig member
    ./scripts/manage_multisig.sh request_and_confirm add_native $TO_CHAIN ${MEMBERS[1]}
    
    # the request ID can be obtained from the last line of last command's output
    REQUEST_ID=
    
    # confirm the request by another member
    ./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}
    
    # if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
    # ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
    
    # view the token list to check if the chain ID is added successfully
    ./scripts/manage_native_token.sh list
```


**3. Transfer mcs/ft/native token to another blockchain through MCS service**

Transfer mcs token to another blockchain:

```shell
    FROM="map001.testnet"  # sender account ID on NEAR blockchain
    TO="[46,120,72,116,221,179,44,215,151,93,104,86,91,80,148,18,165,181,25,244]" # address 0x2E784874ddB32cD7975D68565b509412A5B519F4 
                                                                                  # on target blockchain
    TO_CHAIN=212 # to chain ID
    AMOUNT=100000000000000000000000
    MCS_TOKEN="mcs_token_0".$MCS_ACCOUNT  # mcs token account ID
    
    # get the token balance of the sender
    ./scripts/manage_mcs_token.sh balance $MCS_TOKEN $FROM
    
    # transfer mcs token to receiver on target chain, make sure sender has enough token
    ./scripts/manage_mcs_token.sh transfer $MCS_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
    
    # get the token balance of the sender to check if the token was transferred out successfully
    ./scripts/manage_mcs_token.sh balance $MCS_TOKEN $FROM
```

Transfer ft token to another blockchain:
```shell
    FROM="map001.testnet"
    TO="[46,120,72,116,221,179,44,215,151,93,104,86,91,80,148,18,165,181,25,244]"
    TO_CHAIN=212
    AMOUNT=100000000000000000000000
    FT_TOKEN="wrap.testnet"  # ft token account ID
    
    # get the token balance of the sender
    ./scripts/manage_ft_token.sh balance $FT_TOKEN $FROM
    
    # transfer ft token to receiver on target chain, make sure sender has enough token
    ./scripts/manage_ft_token.sh transfer $FT_TOKEN $TO_CHAIN $FROM $TO $AMOUNT
    
    # get the token balance of the sender to check if the token was transferred out successfully
    ./scripts/manage_ft_token.sh balance $FT_TOKEN $FROM
```

Transfer native token to another blockchain:
```shell
    FROM="map001.testnet"
    TO="[46,120,72,116,221,179,44,215,151,93,104,86,91,80,148,18,165,181,25,244]"
    TO_CHAIN=212
    AMOUNT=100000000000000000000000
    
    # get the token balance of the sender
    ./scripts/manage_native_token.sh balance $FROM
    
    # transfer native token to receiver on target chain, make sure sender has enough token
    ./scripts/manage_native_token.sh transfer $TO_CHAIN $FROM $TO $AMOUNT
    
    # get the token balance of the sender to check if the token was transferred out successfully
    ./scripts/manage_native_token.sh balance $FROM
```

## Upgrade the contracts

The mcs contract and mcs token contract can be upgraded through multisig contract.

### 1. Upgrade mcs contract

**Before upgrading mcs contract, everything (transfer in, transfer out, deposit out...) should be paused.**

```shell
PAUSED_MASK=63  # pause everything

# request to pause everything by multisig member
./scripts/manage_multisig.sh request_and_confirm set_paused $PAUSED_MASK ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

**Then upgrade the mcs contract code.**

The first multisig member should use **[mcs upgrade tool](https://github.com/PandaRR007/mcs-upgrade-tool)** to add request and confirm.

The tool output contains a link to the transaction detail. You can get the request ID from the NEAR explorer.

Other multisig member can confirm and execute the request using below command:

```shell
# the request ID can be obtained from the transaction detail in NEAR explorer
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

**Set the mcs contract state if new state is added to the contract struct.**

E.g, if "map_chain_id" is added, set it using below command:

```shell
MAP_CHAIN_ID="22776"  # MAP chain ID

# request to set new map light client account by multisig member
./scripts/manage_multisig.sh request_and_confirm map_chain_id $MAP_CHAIN_ID ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
#./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

**Finally, unpause everything.**

```shell
PAUSED_MASK=0  # unpause everything

# request to unpause everything by multisig member
./scripts/manage_multisig.sh request_and_confirm set_paused $PAUSED_MASK ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```


### 2. Upgrade mcs token contract

**NOTE**: currently the script works on MacOS only.
```shell
MCS_TOKEN_WASM_FILE=/path/to/mcs/token/contract  # new mcs token contract wasm file
MCS_TOKEN="mcs_token_0".$MCS_ACCOUNT

# request to upgrade mcs token contract by multisig member
./scripts/manage_multisig.sh request_and_confirm upgrade_mcs_token $MCS_TOKEN $MCS_TOKEN_WASM_FILE ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

### 3. Set new MAP light client contract account

The MCS contract supports updating the MAP light client contract account to a new one if the old one is deprecated.

**Before setting new client, the transfer in function should be paused.**

```shell
PAUSED_MASK=2  # pause transfer in

# request to pause transfer in by multisig member
./scripts/manage_multisig.sh request_and_confirm set_paused $PAUSED_MASK ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

**Then set the new client account.**

```shell
NEW_CLIENT_ACCOUNT="new_client1.testnet"  # new MAP light client account ID

# request to set new map light client account by multisig member
./scripts/manage_multisig.sh request_and_confirm set_client $NEW_CLIENT_ACCOUNT ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
#./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

**Finally, unpause transfer in function.**

```shell
PAUSED_MASK=0  # unpause everything

# request to unpause everything by multisig member
./scripts/manage_multisig.sh request_and_confirm set_paused $PAUSED_MASK ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

### 4. Upgrade multisig contract

**NOTE**: currently the script works on MacOS only.
```shell
MULTISIG_WASM_FILE=/path/to/multisig/contract  # new multisig contract wasm file

# request to upgrade multisig contract by multisig member
./scripts/manage_multisig.sh request_and_confirm upgrade_multisig $MULTISIG_WASM_FILE ${MEMBERS[1]}
    
# the request ID can be obtained from the last line of last command's output
REQUEST_ID=
    
# confirm the request by another member
./scripts/manage_multisig.sh confirm $REQUEST_ID ${MEMBERS[2]}

# if the request is not executed because of the time lock, anyone can execute it after REQUEST_LOCK time
# ./scripts/manage_multisig.sh execute $REQUEST_ID $MASTER_ACCOUNT
```

## Testing
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