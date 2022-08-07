
map chain light client contract, will be deployed on near blockchain.


# How to build?

```shell
cargo build --workspace --target wasm32-unknown-unknown --release
```

The mcs contract wasm file "mcs.wasm" and mcs token contract wasm file "mcs_token.wasm" will be generated in ./target/wasm32-unknown-unknown/release.

# How to run unit testing?

```shell
cargo test --workspace --lib
```

# How to run integration testing?

```shell
# set below environment before run tests
export NEAR_SANDBOX_BIN_PATH="/path/to/near/sandbox/bin"

cargo test
```