#!/bin/bash
set -e

KATHERINE_CONTRACT_ADDRESS=dev-1651597684194-74912828106416
METAPOOL_CONTRACT_ADDRESS=dev-1651597642819-66568460980904
SUPPORTER_ID="kate_supporter.testnet"

# Deploy contract to Testnet
# NEAR_ENV=testnet near deploy -f --wasmFile res/katherine_fundraising_contract.wasm --accountId $CONTRACT_NAME

# # NEAR_ENV=testnet near call $CONTRACT_NAME process_kickstarter '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# # NEAR_ENV=testnet near call $CONTRACT_NAME internal_activate_kickstarter '{"kickstarter_id": 0, "goal_id": 0}' --accountId $SUPPORTER_ID --gas=75000000000000

# NEAR_ENV=testnet near call $CONTRACT_NAME unfreeze_kickstarter_funds '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000


# NEAR_ENV=testnet near call $CONTRACT_NAME withdraw_all '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# NEAR_ENV=testnet near view $CONTRACT_NAME get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID
# NEAR_ENV=testnet near view $CONTRACT_NAME get_supporter_estimated_stnear '{"supporter_id": "'$SUPPORTER_ID'", "kickstarter_id": 0, "st_near_price": "1795993457215512332027729"}' --accountId $SUPPORTER_ID

# NEAR_ENV=testnet near call $CONTRACT_NAME unfreeze_kickstarter_funds '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# NEAR_ENV=testnet near view $CONTRACT_NAME get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID

# ---- 1
# NEAR_ENV=testnet near deploy -f --wasmFile res/katherine_fundraising_contract.wasm --accountId $KATHERINE_CONTRACT_ADDRESS
# NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS withdraw_stnear_interest '{"kickstarter_id": 0}' --accountId kate_kickstarter_owner.testnet --gas 300000000000000
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "kate_kickstarter_owner.testnet"}' --accountId kate_kickstarter_owner.testnet

# NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_detailed_list '{"supporter_id": "'$SUPPORTER_ID'", "st_near_price": "'$(date +%s)000000000000000'", "from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID

# NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS unfreeze_kickstarter_funds '{"kickstarter_id": 0}' --accountId kate_kickstarter_owner.testnet --gas 300000000000000
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID
# NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS withdraw_all '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID --gas 300000000000000
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID


# NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID


### Paradox
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $KATHERINE_CONTRACT_ADDRESS
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

# NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer '{"receiver_id": "'$SUPPORTER_ID'", "amount": "3424179303366111364"}' --accountId $KATHERINE_CONTRACT_ADDRESS --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $KATHERINE_CONTRACT_ADDRESS
# NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

#### KATHERINE BETATEST!!!!
# NEAR_ENV=testnet near view v0_1_6.katherine_fundraising.testnet get_project_details '{"kickstarter_id": 0}' --accountId $SUPPORTER_ID
NEAR_ENV=testnet near call v0_1_6.katherine_fundraising.testnet process_kickstarter '{"kickstarter_id": 1}' --accountId $SUPPORTER_ID --gas 300000000000000
NEAR_ENV=testnet near view v0_1_6.katherine_fundraising.testnet get_project_details '{"kickstarter_id": 1}' --accountId $SUPPORTER_ID

### EVALUATE PROJECT

NEAR_ENV=mainnet near view v1.metayield.near get_project_details '{"kickstarter_id": 0}' --accountId jomsox.near
NEAR_ENV=mainnet near view v1.metayield.near get_kickstarters_to_process '{"from_index": 0, "limit": 10}' --accountId jomsox.near

NEAR_ENV=mainnet near call v1.metayield.near get_project_details '{"kickstarter_id": 0}' --accountId jomsox.near