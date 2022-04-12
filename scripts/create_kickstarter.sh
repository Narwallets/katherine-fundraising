#!/bin/bash
set -e

# Contract
CONTRACT_NAME="v01.katherine_fundraising.testnet"
METAPOOL_CONTRACT_ADDRESS="meta-v2.pool.testnet"
KATHERINE_OWNER_ID="v01.katherine_fundraising.testnet"
KATHERINE_MIN_DEPOSIT_AMOUNT=2
KATHERINE_FEE_PERCENT=100
YOCTO_UNITS="000000000000000000000000"

# Kickstarter
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="Super_Duper_Project4"
KICKSTARTER_SLUG="super-duper-project4"
KICKSTARTER_OWNER_ID="imcsk8.testnet"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 864000))
#KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 60000))
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 2592000000))
#KICKSTARTER_TOKEN_ADDRESS="dev-1648794972127-11189653369055"
KICKSTARTER_TOKEN_ADDRESS="v01.chaverocoin.testnet"
KICKSTARTER_ID="3"
export NEAR_ENV=testnet


if [[ "$1" != "" ]]; then
    KICKSTARTER_NAME=$1
    KICKSTARTER_NAME="${KICKSTARTER_NAME// /_}"
    KICKSTARTER_SLUG=$(echo $KICKSTARTER_NAME | tr '[:upper:]' '[:lower:]')
else
    echo "Missing kickstarter name argument"
    exit
fi

# Goals
GOAL_1_NAME="Goal_Number_1"
GOAL_1_DESIRED_AMOUNT="10"$YOCTO_UNITS
GOAL_1_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 2592000000))
GOAL_1_END_DATE=$(($GOAL_1_CLIFF_DATE + 2592000000))
GOAL_1_UNFREEZE_DATE=$GOAL_1_END_DATE
GOAL_1_TOKENS_TO_RELEASE="0"$YOCTO_UNITS

# Goals
GOAL_2_NAME="Goal_Number_2"
GOAL_2_DESIRED_AMOUNT="10"$YOCTO_UNITS
GOAL_2_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 2593000000))
GOAL_2_END_DATE=$(($GOAL_2_CLIFF_DATE + 2595000000))
GOAL_2_UNFREEZE_DATE=$GOAL_2_END_DATE
GOAL_2_TOKENS_TO_RELEASE="0"$YOCTO_UNITS

# Goals
GOAL_3_NAME="Goal_Number_3"
GOAL_3_DESIRED_AMOUNT="10"$YOCTO_UNITS
GOAL_3_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 2596000000))
GOAL_3_END_DATE=$(($GOAL_3_CLIFF_DATE + 2597000000))
GOAL_3_UNFREEZE_DATE=$GOAL_3_END_DATE
GOAL_3_TOKENS_TO_RELEASE="0"$YOCTO_UNITS


echo "Creating a Kickstarter: ${KICKSTARTER_NAME} with ${KICKSTARTER_SLUG}"
near call $CONTRACT_NAME create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$KICKSTARTER_TOKEN_ADDRESS'"}' --accountId $KATHERINE_OWNER_ID


KICKSTARTER_ID=$(NEAR_ENV=testnet near call $CONTRACT_NAME get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

echo "Kickstarter ID: ${KICKSTARTER_ID}"

# Create goal
echo "Creating Goal #2"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_2_NAME'", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_2_UNFREEZE_DATE', "tokens_to_release": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_2_CLIFF_DATE', "end_timestamp": '$GOAL_2_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID

# Create goal 3
echo "Creating Goal #3"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_3_NAME'", "desired_amount": "'$GOAL_3_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_3_UNFREEZE_DATE', "tokens_to_release": "'$GOAL_3_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_3_CLIFF_DATE', "end_timestamp": '$GOAL_3_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID
