 MAP chain light client contract, will be deployed on NEAR blockchain.

# Pre-requisites

Make sure Rust development environment is installed and configured. 

See [here](https://www.rust-lang.org/tools/install) about how to install.

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