#!/bin/bash
set -e

source meta_yield.conf

# Kickstarter
NOW_IN_MILLISECS="1651449600000" # 05/02/2022 0:00:00
PROJECT_NAME="PembRock"
PROJECT_SLUG="pembrock"
PROJECT_OWNER_ID="pembrock_betatest.testnet"
PROJECT_OPEN_DATE="1651449600000" # 05/02/2022 0:00:00
PROJECT_CLOSE_DATE="1651622400000" # 05/04/2022 0:00:00
PROJECT_TOKEN_ADDRESS="token_pembrock_betatest.testnet"
PROJECT_ID="0"
DEPOSITS_HARD_CAP="100000${YOCTO_UNITS}"
MAX_TOKENS_TO_RELEASE="10${YOCTO_UNITS}"
TOKEN_TOTAL_SUPPLY="5000000${YOCTO_UNITS}"
export NEAR_ENV=testnet

# Needed ptoken supply form kickstarter: DEPOSITS_HARD_CAP * MAX_TOKENS_TO_RELEASE * 1.02
#PTOKENS_FOR_KICKSTARTER=$(bc <<< "100000 * 10 * 1.02")

# Goals
GOAL_1_NAME="Goal_Number_1"
GOAL_1_DESIRED_AMOUNT="15000"$YOCTO_UNITS
GOAL_1_CLIFF_DATE="1651708800000"
GOAL_1_END_DATE="1651881600000"
GOAL_1_UNFREEZE_DATE=$GOAL_1_END_DATE
GOAL_1_TOKENS_TO_RELEASE="6"$YOCTO_UNITS

# Goals
GOAL_2_NAME="Goal_Number_2"
GOAL_2_DESIRED_AMOUNT="25000"$YOCTO_UNITS
GOAL_2_CLIFF_DATE="1651708800000"
GOAL_2_END_DATE="1651881600000"
GOAL_2_UNFREEZE_DATE=$GOAL_2_END_DATE
GOAL_2_TOKENS_TO_RELEASE="8"$YOCTO_UNITS


if [[ "${1}" != "" ]]; then
    CONFIGURATION_FILE=$1
fi

if [[ "${PROJECT_CHECK}" != "no" ]]; then
	echo "Using project configuration file ${CONFIGURATION_FILE} with the following options: "
	echo "-----------------------------------------------------------------"
	echo
	cat $CONFIGURATION_FILE
	echo "-----------------------------------------------------------------"
	echo 'If you do not wish to check the project set the PROJECT_CHECK variable to "no"'

	echo "Are the project settings correct? "
	select yn in "Yes" "No"; do
    	case $yn in
        	Yes ) break;;
        	No ) echo "Please update your project settings"; exit;;
    	esac
	done
fi

source $CONFIGURATION_FILE

echo "Creating a Kickstarter: ${PROJECT_NAME} with ${PROJECT_SLUG}"
near call $CONTRACT_NAME create_kickstarter '{"name": "'${PROJECT_NAME}'", "slug": "'$PROJECT_SLUG'", "owner_id": "'$PROJECT_OWNER_ID'", "open_timestamp": '$PROJECT_OPEN_DATE', "close_timestamp": '$PROJECT_CLOSE_DATE', "token_contract_address": "'$PROJECT_TOKEN_ADDRESS'" ,"deposits_hard_cap": "'${DEPOSITS_HARD_CAP}'", "max_tokens_to_release_per_stnear": "'${MAX_TOKENS_TO_RELEASE}'"}' --accountId $KATHERINE_OWNER_ID

PROJECT_ID=$(NEAR_ENV=testnet near call $CONTRACT_NAME get_kickstarter_id_from_slug '{"slug": "'$PROJECT_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

echo "Project ID: ${PROJECT_ID}"

#if [[ "${NEAR_ENV}" == "testnet" ]]; then
    #echo "Deploying ${PROJECT_NAME} test token: "
    #NEAR_ENV=testnet near deploy --wasmFile ../res/test_p_token.wasm --initFunction new_default_meta --initArgs '{"owner_id": "'${PROJECT_OWNER_ID}'", "total_supply": "'${TOKEN_TOTAL_SUPPLY}'"}' --accountId $PROJECT_TOKEN_ADDRESS
    #echo "Registering katherine and kickstarter owners to test token contract: ${PROJECT_TOKEN_ADDRESS}"
    #NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS register_account '{"account_id": "'$KATHERINE_OWNER_ID'"}' --accountId $KATHERINE_OWNER_ID
    #NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS register_account '{"account_id": "'$PROJECT_OWNER_ID'"}' --accountId $PROJECT_OWNER_ID
    #NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS register_account '{"account_id": "'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME
    #echo "------------------ Sending Test PembRock tokens to the kickstarter"
    #NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS ft_transfer '{"receiver_id": "'$PROJECT_OWNER_ID'", "amount": "'210000$YOCTO_UNITS'"}' --accountId $PROJECT_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

    # Sending pTokens to Kickstarter
    #echo "------------------ Sending Test PembRock tokens to the contract"
    #NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS ft_transfer_call '{"receiver_id": "'$CONTRACT_NAME'", "amount": "'1200000$YOCTO_UNITS'", "msg": "'$PROJECT_ID'"}' --accountId $PROJECT_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS
#fi


# Create goal 1
echo "Creating Goal #1"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$PROJECT_ID', "name": "'$GOAL_1_NAME'", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_1_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_1_CLIFF_DATE', "end_timestamp": '$GOAL_1_END_DATE', "reward_installments": 5}' --accountId $PROJECT_OWNER_ID

echo "Created goal #1 for kickstarter: ${PROJECT_ID}"

# Create goal 2
echo "Creating Goal #2"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$PROJECT_ID', "name": "'$GOAL_2_NAME'", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_2_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_2_CLIFF_DATE', "end_timestamp": '$GOAL_2_END_DATE', "reward_installments": 5}' --accountId $PROJECT_OWNER_ID

echo "Created goal #2 for kickstarter: ${PROJECT_ID}"

# Create goal 3
echo "Creating Goal #3"
near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$PROJECT_ID', "name": "'$GOAL_3_NAME'", "desired_amount": "'$GOAL_3_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_3_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_3_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_3_CLIFF_DATE', "end_timestamp": '$GOAL_3_END_DATE', "reward_installments": 5}' --accountId $PROJECT_OWNER_ID

echo "Created goal #3 for Project: ${PROJECT_ID}"
