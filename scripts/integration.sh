#!/bin/bash
set -e

METAPOOL_CONTRACT_ADDRESS="meta-v2.pool.testnet"
KATHERINE_OWNER_ID="huxley.testnet"
KATHERINE_MIN_DEPOSIT_AMOUNT=2
KATHERINE_FEE_PERCENT=100
YOCTO_UNITS="000000000000000000000000"

# Deploy contract to Testnet
rm -rf neardev/
NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": "'${KATHERINE_OWNER_ID}'", "min_deposit_amount": '${KATHERINE_MIN_DEPOSIT_AMOUNT}', "metapool_contract_address": "'${METAPOOL_CONTRACT_ADDRESS}'", "katherine_fee_percent": '${KATHERINE_FEE_PERCENT}'}'

export $(grep -v '^#' neardev/dev-account.env | xargs)
echo '------------------ Contract deployed to: '$CONTRACT_NAME

# Create a Kickstarter project
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="The_Best_Project_Ever"
KICKSTARTER_SLUG="the-best-project-ever"
KICKSTARTER_OWNER_ID="jomsox.testnet"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 120000))
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
GOAL_1_DESIRED_AMOUNT="1"$YOCTO_UNITS
GOAL_1_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_1_END_DATE=$(($GOAL_1_CLIFF_DATE + 60000))
GOAL_1_UNFREEZE_DATE=$GOAL_1_END_DATE
GOAL_1_TOKENS_TO_RELEASE="0"$YOCTO_UNITS
echo "------------------ Creating Goal #1"
NEAR_ENV=testnet near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_1_NAME'", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_1_UNFREEZE_DATE', "tokens_to_release": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_1_CLIFF_DATE', "end_timestamp": '$GOAL_1_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID
echo "TOTAL GOALS:"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter_total_goals '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

GOAL_2_NAME="Goal_Number_2"
GOAL_2_DESIRED_AMOUNT="2000000"$YOCTO_UNITS
GOAL_2_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_2_END_DATE=$(($GOAL_2_CLIFF_DATE + 60000))
GOAL_2_UNFREEZE_DATE=$GOAL_2_END_DATE
GOAL_2_TOKENS_TO_RELEASE="200000000"$YOCTO_UNITS
echo "------------------ Creating Goal #2"
NEAR_ENV=testnet near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_2_NAME'", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_2_UNFREEZE_DATE', "tokens_to_release": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_2_CLIFF_DATE', "end_timestamp": '$GOAL_2_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID
echo "TOTAL GOALS:"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter_total_goals '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# Deleting last goal
echo "------------------ Deleting Goal #2"
NEAR_ENV=testnet near call $CONTRACT_NAME delete_last_goal '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KICKSTARTER_OWNER_ID
echo "TOTAL GOALS:"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter_total_goals '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# Get/view the Kickstarter goal
echo "------------------ Get Goal by Id"
GOAL_1_ID=0
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarter_goal '{"kickstarter_id": '$KICKSTARTER_ID', "goal_id": '$GOAL_1_ID'}' --accountId $KATHERINE_OWNER_ID

# FRONTEND CALL: get_active_projects
echo "------------------ FRONTEND: Get Active Projects"
ACTIVE_FROM_IX=0
ACTIVE_TO_IX=10
NEAR_ENV=testnet near view $CONTRACT_NAME get_active_projects '{"from_index": '$ACTIVE_FROM_IX', "limit": '$ACTIVE_TO_IX'}' --accountId $KATHERINE_OWNER_ID

# Sending stnear tokens to Kickstarter
NOW_IN_SECS=$(date +%s)
OPEN_DATE_IN_SECS=$(($KICKSTARTER_OPEN_DATE / 1000))
WAITING_SECONDS=$(($OPEN_DATE_IN_SECS - $NOW_IN_SECS))
echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
sleep $WAITING_SECONDS
SUPPORTER_ID="aldous.testnet"
SUPPORTER_AMOUNT="1"$YOCTO_UNITS
SUPPORTER_MSG="0"
TOTAL_PREPAID_GAS=300000000000000
NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$CONTRACT_NAME'", "amount": "'$SUPPORTER_AMOUNT'", "msg": "'$SUPPORTER_MSG'"}' --accountId $SUPPORTER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Get/view the supporter deposit
echo "------------------ Supporter deposit!"
NEAR_ENV=testnet near view $CONTRACT_NAME get_supporter_total_deposit_in_kickstarter '{"supporter_id": "'$SUPPORTER_ID'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# FRONTEND CALL: get_active_projects
echo "------------------ FRONTEND: Get Active Projects"
ACTIVE_FROM_IX=0
ACTIVE_TO_IX=10
NEAR_ENV=testnet near view $CONTRACT_NAME get_active_projects '{"from_index": '$ACTIVE_FROM_IX', "limit": '$ACTIVE_TO_IX'}' --accountId $KATHERINE_OWNER_ID

# FRONTEND CALL: get_project_details
echo "------------------ FRONTEND: Get Project Details"
NEAR_ENV=testnet near view $CONTRACT_NAME get_project_details '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# Withdraw stnear tokens from Kickstarter
echo "------------------ Supporter withdraw FIRST HALF before freeze!"
SUPPORTER_AMOUNT_HALF="500000000000000000000000"
NEAR_ENV=testnet near call $CONTRACT_NAME withdraw '{"amount": "'$SUPPORTER_AMOUNT_HALF'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas $TOTAL_PREPAID_GAS

# Get/view the supporter deposit
echo "------------------ Support"
NEAR_ENV=testnet near view $CONTRACT_NAME get_supporter_total_deposit_in_kickstarter '{"supporter_id": "'$SUPPORTER_ID'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID

# Withdraw stnear tokens from Kickstarter
echo "------------------ Supporter withdraw SECOND HALF before freeze!"
NEAR_ENV=testnet near call $CONTRACT_NAME withdraw '{"amount": "'$SUPPORTER_AMOUNT_HALF'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas $TOTAL_PREPAID_GAS

# Get/view the supporter deposit
echo "------------------ Supporter IS NOW EMPTY!"
{
    NEAR_ENV=testnet near view $CONTRACT_NAME get_supporter_total_deposit_in_kickstarter '{"supporter_id": "'$SUPPORTER_ID'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KATHERINE_OWNER_ID
} || {
    echo "ERROR EXPECTED! Supporter is EMPTY!"
}

# Supporter deposits Again
echo "------------------ Supporter deposits again to freeze funds!"
NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$CONTRACT_NAME'", "amount": "'$SUPPORTER_AMOUNT'", "msg": "'$SUPPORTER_MSG'"}' --accountId $SUPPORTER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Try to withdraw when freezed funds
NOW_IN_SECS=$(date +%s)
CLOSE_DATE_IN_SECS=$(($KICKSTARTER_CLOSE_DATE / 1000))
WAITING_SECONDS=$(($CLOSE_DATE_IN_SECS - $NOW_IN_SECS))
echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
sleep $(($WAITING_SECONDS + 1))

# ROBOT
echo "------------------ ROBOT: Get Projects"
NEAR_ENV=testnet near view $CONTRACT_NAME get_kickstarters_to_process '{"from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID

echo "------------------ ROBOT: Processing kickstarter"
NEAR_ENV=testnet near call $CONTRACT_NAME process_kickstarter '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas 300000000000000

# echo "------------------ Supporter is trying to withdraw before unfreeze!"
# {
#     NEAR_ENV=testnet near call $CONTRACT_NAME withdraw '{"amount": "'$SUPPORTER_AMOUNT'", "kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas $TOTAL_PREPAID_GAS
# } || {
#     echo "ERROR EXPECTED! Supporter is not allow to withdraw before unfreeze!"
# }

# # Try to withdraw when unfreezed funds
# NOW_IN_SECS=$(date +%s)
# CLOSE_DATE_IN_SECS=$(($GOAL_1_UNFREEZE_DATE / 1000))
# WAITING_SECONDS=$(($CLOSE_DATE_IN_SECS - $NOW_IN_SECS))
# echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
# sleep $WAITING_SECONDS
# echo "------------------ ROBOT: Unfreezing funds!"
# NEAR_ENV=testnet near call $CONTRACT_NAME unfreeze_kickstarter_funds '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas 300000000000000

# echo "------------------ Supporter if FINALLY withdrawing all his funds!"
# NEAR_ENV=testnet near call $CONTRACT_NAME withdraw_all '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas $TOTAL_PREPAID_GAS
