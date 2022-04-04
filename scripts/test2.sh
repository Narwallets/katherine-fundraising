#!/bin/bash
set -e

CONTRACT_NAME=dev-1649097934883-14139568838962
SUPPORTER_ID="aldous.testnet"

# Deploy contract to Testnet
NEAR_ENV=testnet near deploy -f --wasmFile res/katherine_fundraising_contract.wasm --accountId $CONTRACT_NAME

# NEAR_ENV=testnet near call $CONTRACT_NAME process_kickstarter '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
NEAR_ENV=testnet near call $CONTRACT_NAME internal_activate_kickstarter '{"kickstarter_id": 0, "goal_id": 0}' --accountId $SUPPORTER_ID --gas=75000000000000