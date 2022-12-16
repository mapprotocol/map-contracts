#!/usr/bin/env bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR

RFLAGS='-C link-arg=-s'

cd $SCRIPT_DIR/../mos-token
echo "start to build mos-token"
RUSTFLAGS=$RFLAGS cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/../map-ominichain-service
echo "start to build mos"
RUSTFLAGS=$RFLAGS cargo build --target wasm32-unknown-unknown --release

cd $SCRIPT_DIR/..
echo "start to build other packages"
RUSTFLAGS=$RFLAGS cargo build --workspace --exclude mos-token --exclude map-ominichain-service --target wasm32-unknown-unknown --release

mkdir $RES_DIR
cp $RELEASE_DIR/*.wasm $RES_DIR