#!/usr/bin/env bash

set -e
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR

export RUSTFLAGS='-C link-arg=-s'

cd $SCRIPT_DIR/../multisig
echo "start to build multisig"
cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/../multisig-factory
echo "start to build multisig-factory"
cargo build --target wasm32-unknown-unknown --release

mkdir $RES_DIR
cp $RELEASE_DIR/*.wasm $RES_DIR