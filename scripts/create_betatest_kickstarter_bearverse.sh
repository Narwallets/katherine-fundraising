#!/bin/bash
set -e

# Contract
CONTRACT_NAME="v0_1_6.katherine_fundraising.testnet"
METAPOOL_CONTRACT_ADDRESS="meta-v2.pool.testnet"
KATHERINE_OWNER_ID="katherine_fundraising.testnet"
KATHERINE_MIN_DEPOSIT_AMOUNT="1000000000000000000000000"
KATHERINE_FEE_PERCENT=200
YOCTO_UNITS="000000000000000000000000"
TOTAL_PREPAID_GAS=300000000000000

# Kickstarter
NOW_IN_MILLISECS="1651449600000" # 05/02/2022 0:00:00
KICKSTARTER_NAME="BearVerse"
KICKSTARTER_SLUG="bearverse"
KICKSTARTER_OWNER_ID="bearverse_betatest.testnet"
KICKSTARTER_OPEN_DATE="1651536000000" # 03/02/2022 0:00:00
KICKSTARTER_CLOSE_DATE="1651622400000" # 04/04/2022 0:00:00
KICKSTARTER_TOKEN_ADDRESS="token_bearverse_betatest.testnet"
KICKSTARTER_ID="0"
DEPOSITS_HARD_CAP="50000${YOCTO_UNITS}"
MAX_TOKENS_TO_RELEASE="15${YOCTO_UNITS}"
TOKEN_TOTAL_SUPPLY="5000000${YOCTO_UNITS}"
export NEAR_ENV=testnet

# Goals
GOAL_1_NAME="Goal_Number_1"
GOAL_1_DESIRED_AMOUNT="10000"$YOCTO_UNITS
GOAL_1_CLIFF_DATE="1651708800000"
GOAL_1_END_DATE="1651881600000"
GOAL_1_UNFREEZE_DATE=$GOAL_1_END_DATE
GOAL_1_TOKENS_TO_RELEASE="5"$YOCTO_UNITS

# Goals
GOAL_2_NAME="Goal_Number_2"
GOAL_2_DESIRED_AMOUNT="20000"$YOCTO_UNITS
GOAL_2_CLIFF_DATE="1651708800000"
GOAL_2_END_DATE="1651881600000"
GOAL_2_UNFREEZE_DATE=$GOAL_2_END_DATE
GOAL_2_TOKENS_TO_RELEASE="7"$YOCTO_UNITS


echo "Deploying BearVerse test token: "
NEAR_ENV=testnet near deploy --wasmFile ../res/test_p_token.wasm --initFunction new_default_meta --initArgs '{"owner_id": "'${KICKSTARTER_OWNER_ID}'", "total_supply": "'${TOKEN_TOTAL_SUPPLY}'"}' --accountId $KICKSTARTER_TOKEN_ADDRESS

echo "Registering katherine and kickstarter owners to test token contract: ${KICKSTARTER_TOKEN_ADDRESS}"
NEAR_ENV=testnet near call $KICKSTARTER_TOKEN_ADDRESS register_account '{"account_id": "'$KATHERINE_OWNER_ID'"}' --accountId $KATHERINE_OWNER_ID
NEAR_ENV=testnet near call $KICKSTARTER_TOKEN_ADDRESS register_account '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID
NEAR_ENV=testnet near call $KICKSTARTER_TOKEN_ADDRESS register_account '{"account_id": "'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME

echo "Creating a Kickstarter: ${KICKSTARTER_NAME} with ${KICKSTARTER_SLUG}"
near call $CONTRACT_NAME create_kickstarter '{"name": "'${KICKSTARTER_NAME}'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$KICKSTARTER_TOKEN_ADDRESS'" ,"deposits_hard_cap": "'${DEPOSITS_HARD_CAP}'", "max_tokens_to_release_per_stnear": "'${MAX_TOKENS_TO_RELEASE}'"}' --accountId $KATHERINE_OWNER_ID

KICKSTARTER_ID=$(NEAR_ENV=testnet near call $CONTRACT_NAME get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

echo "Kickstarter ID: ${KICKSTARTER_ID}"

echo "------------------ Sending Test BearVerse tokens to the kickstarter"
NEAR_ENV=testnet near call $KICKSTARTER_TOKEN_ADDRESS ft_transfer '{"receiver_id": "'$KICKSTARTER_OWNER_ID'", "amount": "'210000$YOCTO_UNITS'"}' --accountId $KICKSTARTER_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Sending pTokens to Kickstarter
echo "------------------ Sending Test BearVerse tokens to the contract"
NEAR_ENV=testnet near call $KICKSTARTER_TOKEN_ADDRESS ft_transfer_call '{"receiver_id": "'$CONTRACT_NAME'", "amount": "'8000000$YOCTO_UNITS'", "msg": "'$KICKSTARTER_ID'"}' --accountId $KICKSTARTER_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Create goal 1
echo "Creating Goal #1"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_1_NAME'", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_1_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_1_CLIFF_DATE', "end_timestamp": '$GOAL_1_END_DATE', "reward_installments": 5}' --accountId $KICKSTARTER_OWNER_ID

echo "Created one goal for kickstarter: ${KICKSTARTER_ID}"

# Create goal 2
echo "Creating Goal #2"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_2_NAME'", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_2_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_2_CLIFF_DATE', "end_timestamp": '$GOAL_2_END_DATE', "reward_installments": 5}' --accountId $KICKSTARTER_OWNER_ID
