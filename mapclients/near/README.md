# MAP light client

This is MAP blockchain light client contract, which will be deployed on NEAR blockchain. The off-chain program called
maintainer submits the MAP chain block header to the light client contract per epoch and the light client will keep the
validators after verifying it successfully. So that the light client can be used to verify the event happened on 
MAP blockchain.

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
2 wasm files will be generated in directory ./script/res:
1. **map_light_client.wasm**: this is the MAP light client contract
2. **map_client_factory.wasm**: this is the factory contract to deploy and initialize the MAP light client contract and make the MAP light contract account in locked state.

## Deploy the contracts

1. Configure below parameters in ./scripts/config.sh
```shell
MASTER_ACCOUNT=map001.testnet # make sure the account is already created on NEAR blockchain
FACTORY_NAME=fac # the name of map client factory contract to be created, the account ID will be $FACTORY_NAME.$MASTER_ACCOUNT
CLIENT_NAME=client # the name of MAP light client contract to be created, the account ID will be $CLIENT_NAME.$FACTORY_NAME
MAP_RPC_URL=http://3.0.19.66:7445  # the RPC url of MAP blockchain
EPOCH_ID=300  # get the information of this epoch id to initialize the MAP light client contract
```

2. Deploy factory contract and MAP light client contract, and initialize it with below command:
```shell
    ./scripts/deploy.sh
```


## Testing

1. run the unit testing
```shell
cargo test --workspace --lib
```

2. run the integration testing

**NOTE**: Before run the integration testing, make sure **near sandbox** exists on your computer. 
If not, please clone the [nearcore](https://github.com/near/nearcore) project and run "make sandbox" to build it.

```shell
# set below environment before run tests
export NEAR_SANDBOX_BIN_PATH="/path/to/near/sandbox/bin"

cargo test
```