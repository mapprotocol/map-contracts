#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR
RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR


cd $SCRIPT_DIR/..
echo "start to build map light client"
cargo build --workspace --target wasm32-unknown-unknown --release

mkdir $RES_DIR
cp $RELEASE_DIR/map_client_factory.wasm $RES_DIR
cp $RELEASE_DIR/map_light_client.wasm $RES_DIR
