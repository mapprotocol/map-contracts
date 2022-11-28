#!/usr/bin/env bash

set -e
RELEASE=true

usage() {
  echo "Usage: $0 [-t]"
  exit 1
}

while getopts ":t" o; do
    case "${o}" in
        t)
            RELEASE=false
            ;;
        *)
            usage
            ;;
    esac
done

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

RELEASE_DIR=$SCRIPT_DIR/../target/wasm32-unknown-unknown/release
RES_DIR=$SCRIPT_DIR/res

echo "removing old res directory"
echo "rm -rf $RES_DIR"
rm -rf $RES_DIR

export RUSTFLAGS='-C link-arg=-s'

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

cd $SCRIPT_DIR/..
echo "start to build other packages"
cargo build --workspace --exclude mcs-token --exclude --target wasm32-unknown-unknown --release

mkdir $RES_DIR
cp $RELEASE_DIR/*.wasm $RES_DIR