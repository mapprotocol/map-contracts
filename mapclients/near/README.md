
map chain light client contract, will be deployed on near blockchain.


# How to build?

```shell
cargo build --workspace --target wasm32-unknown-unknown --release
```

The contract wasm file "map_light_client.wasm" will be generated in ./target/wasm32-unknown-unknown/release.

# How to run unit testing?

```shell
cargo test --workspace --lib
```

# How to run e2e testing?

```shell
# set below environment before run tests
export NEAR_SANDBOX_BIN_PATH="/path/to/near/sandbox/bin"

cargo test
```