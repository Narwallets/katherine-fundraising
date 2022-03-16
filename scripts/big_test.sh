#!/bin/bash
set -e

METAPOOL_CONTRACT_ADDRESS="meta-v2.pool.testnet"
KATHERINE_OWNER_ID="huxley.testnet"
KATHERINE_MIN_DEPOSIT_AMOUNT=2
KATHERINE_FEE_PERCENT=100
YOCTO_UNITS="000000000000000000000000"

# Deploy contract to Testnet
rm -rf neardev/
NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "'${KATHERINE_OWNER_ID}'", "min_deposit_amount": '${KATHERINE_MIN_DEPOSIT_AMOUNT}', "metapool_contract_address": "'${METAPOOL_CONTRACT_ADDRESS}'", "katherine_fee_percent": '${KATHERINE_FEE_PERCENT}'}'

export $(grep -v '^#' neardev/dev-account.env | xargs)
echo '------------------ Contract deployed to: '$CONTRACT_NAME

# Create a Kickstarter project
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="The_Best_Project_Ever"
KICKSTARTER_SLUG="the-best-project-ever"
KICKSTARTER_OWNER_ID="jomsox.testnet"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 60000))
KICKSTARTER_TOKEN_ADDRESS="meta-v2.pool.testnet"
echo "------------------ Creating a Kickstarter"
NEAR_ENV=testnet near call $CONTRACT_NAME create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$KICKSTARTER_TOKEN_ADDRESS'"}' --accountId $KATHERINE_OWNER_ID

# Get/view the KickstarterId from slug
echo "------------------ Get KickstarterId"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID

# Get/view the Kickstarter from id
KICKSTARTER_ID=0
echo "------------------ Get Kickstarter"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# Create 2 goals
GOAL_1_NAME="Goal_Number_1"
GOAL_1_DESIRED_AMOUNT="1000000"$YOCTO_UNITS
GOAL_1_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_1_END_DATE=$(($GOAL_1_CLIFF_DATE + 60000))
GOAL_1_TOKENS_TO_RELEASE="100000000"$YOCTO_UNITS
echo "------------------ Creating Goal #1"
NEAR_ENV=testnet near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_1_NAME'", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "tokens_to_release": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_1_CLIFF_DATE', "end_timestamp": '$GOAL_1_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID
