#!/bin/bash
set -e

CONTRACT_NAME=dev-1649263589412-13459275688014
SUPPORTER_ID="aldous.testnet"

# Deploy contract to Testnet
# NEAR_ENV=testnet near deploy -f --wasmFile res/katherine_fundraising_contract.wasm --accountId $CONTRACT_NAME

# # NEAR_ENV=testnet near call $CONTRACT_NAME process_kickstarter '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# # NEAR_ENV=testnet near call $CONTRACT_NAME internal_activate_kickstarter '{"kickstarter_id": 0, "goal_id": 0}' --accountId $SUPPORTER_ID --gas=75000000000000

# NEAR_ENV=testnet near call $CONTRACT_NAME unfreeze_kickstarter_funds '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000


# NEAR_ENV=testnet near call $CONTRACT_NAME withdraw_all '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# NEAR_ENV=testnet near view $CONTRACT_NAME get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID
# NEAR_ENV=testnet near view $CONTRACT_NAME get_supporter_estimated_stnear '{"supporter_id": "'$SUPPORTER_ID'", "kickstarter_id": 0, "st_near_price": "1795993457215512332027729"}' --accountId $SUPPORTER_ID

# NEAR_ENV=testnet near call $CONTRACT_NAME unfreeze_kickstarter_funds '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
NEAR_ENV=testnet near view $CONTRACT_NAME get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID

