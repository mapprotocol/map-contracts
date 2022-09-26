#!/usr/bin/env bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source $SCRIPT_DIR/config.sh
echo $SCRIPT_DIR

RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR

cd $SCRIPT_DIR/../../../mapclients/near/contracts
echo "start to build map light client"
cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/../../../mapclients/near/map-client-factory
echo "start to build map light client factory"
cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/../mcs-token
echo "start to build mcs-token"
if $RELEASE; then
  echo "building release mcs token"
  cargo build --target wasm32-unknown-unknown --release
else
  echo "building non-release mcs token"
  cargo build --target wasm32-unknown-unknown --release --no-default-features
fi

cd $SCRIPT_DIR/../map-cross-chain-service
echo "start to build mcs"
cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/../multisig
echo "start to build multisig"
cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/..
echo "start to build other packages"
cargo build --workspace --exclude mcs-token --exclude multisig --target wasm32-unknown-unknown --release

mkdir $RES_DIR
cp $SCRIPT_DIR/../../../mapclients/near/target/wasm32-unknown-unknown/release/*.wasm  $RES_DIR
cp $RELEASE_DIR/*.wasm $RES_DIR