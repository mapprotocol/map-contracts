 MAP chain light client contract, will be deployed on NEAR blockchain.

# Pre-requisites

1. rust

Follow [these instructions](https://doc.rust-lang.org/book/ch01-01-installation.html) for setting up Rust.
Then, add the **wasm32-unknown-unknown** toolchain which enables compiling Rust to Web Assembly (wasm), the low-level language used by the NEAR platform.

```shell
# Get Rust in linux and MacOS
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Add the wasm toolchain
rustup target add wasm32-unknown-unknown
```

2. near-cli

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

# Build the contracts

Run below command to build:

```shell
cargo build --workspace --target wasm32-unknown-unknown --release
```

or you can simply run 
```shell
./scripts/build.sh
```
2 wasm files will be generated in directory ./target/wasm32-unknown-unknown/release:
1. **map_light_client.wasm**: this is the MAP light client contract
2. **map_client_factory.wasm**: this is the factory contract to deploy and initialize the MAP light client contract and make the MAP light contract account in locked state.

# Deploy the contracts
See [here](../../mcs/near/README.md) for details.

# Testing

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