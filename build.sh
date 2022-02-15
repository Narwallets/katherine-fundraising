#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/katherine_fundraising.wasm res/