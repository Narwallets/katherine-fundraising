#!/bin/bash
set -e

# Contract
# KICKSTARTER_OWNER_ID="katherine_for_manza.testnet"
METAPOOL_CONTRACT_ADDRESS="meta-v2.pool.testnet"
KATHERINE_OWNER_ID="katherine_for_manza.testnet"
KATHERINE_MIN_DEPOSIT_AMOUNT="2000000000000"
KATHERINE_FEE_PERCENT=100
YOCTO_UNITS="000000000000000000000000"

KICKSTARTER_OWNER_ID="kate_kickstarter_owner.testnet"
SUPPORTER_ID="kate_supporter.testnet"
PTOKEN_CONTRACT_ADDRESS="katherine_for_manza.testnet"

# NEAR_ENV=testnet near deploy -f --accountId $KICKSTARTER_OWNER_ID --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": "'$KICKSTARTER_OWNER_ID'", "min_deposit_amount": "2000000000000", "metapool_contract_address": "meta-v2.pool.testnet", "katherine_fee_percent": 100 }'

# KICKSTARTER_ID=0
# NOW_IN_MILLISECS=$(($(date +%s) * 1000))
# KICKSTARTER_NAME="The_Best_Project_Ever"
# KICKSTARTER_SLUG="the-best-project-ever-${NOW_IN_MILLISECS}"
# KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
# # Cierre de período de fondeo
# KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 7200000))
# echo "------------------ Creating a Kickstarter"
# NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "deposits_hard_cap": "'9$YOCTO_UNITS'", "max_tokens_to_release_per_stnear": "'2$YOCTO_UNITS'"}' --accountId $KICKSTARTER_OWNER_ID

# KICKSTARTER_ID=$(NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

# # Create 2 goals
# GOAL_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
# GOAL_END_DATE=$(($GOAL_CLIFF_DATE + 60000))
# GOAL_UNFREEZE_DATE=$GOAL_END_DATE

# GOAL_1_DESIRED_AMOUNT="5"$YOCTO_UNITS
# GOAL_1_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
# echo "------------------ Creating Goal #1"
# NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Silver", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE', "reward_installments": 12}' --accountId $KICKSTARTER_OWNER_ID

# GOAL_2_DESIRED_AMOUNT="8"$YOCTO_UNITS
# GOAL_2_TOKENS_TO_RELEASE="2"$YOCTO_UNITS
# echo "------------------ Creating Goal #2"
# NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Gold", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE', "reward_installments": 12}' --accountId $KICKSTARTER_OWNER_ID


# Create a Kickstarter project
KICKSTARTER_ID=1
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="The_Second_Best_Project_Ever"
KICKSTARTER_SLUG="the-best-project-ever-${NOW_IN_MILLISECS}"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 172800000))
echo "------------------ Creating a Kickstarter"
NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "deposits_hard_cap": "'5$YOCTO_UNITS'", "max_tokens_to_release_per_stnear": "'1$YOCTO_UNITS'"}' --accountId $KICKSTARTER_OWNER_ID

KICKSTARTER_ID=$(NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

# Create 2 goals
GOAL_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_END_DATE=$(($GOAL_CLIFF_DATE + 60000))
GOAL_UNFREEZE_DATE=$GOAL_END_DATE

GOAL_1_DESIRED_AMOUNT="2"$YOCTO_UNITS
GOAL_1_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
echo "------------------ Creating Goal #1"
NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Silver", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE', "reward_installments": 12}' --accountId $KICKSTARTER_OWNER_ID

GOAL_2_DESIRED_AMOUNT="4"$YOCTO_UNITS
GOAL_2_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
echo "------------------ Creating Goal #2"
NEAR_ENV=testnet near call $KICKSTARTER_OWNER_ID create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Gold", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE', "reward_installments": 12}' --accountId $KICKSTARTER_OWNER_ID