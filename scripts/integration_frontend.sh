#!/bin/bash
set -e

# Run `make integration-meta-pool` to generate a fresh new metapool and katherine contract.

METAPOOL_CONTRACT_ADDRESS=$(head -n1 ./neardev_metapool/dev-account)
KATHERINE_CONTRACT_ADDRESS=$(head -n1 ./neardev/dev-account)
PTOKEN_CONTRACT_ADDRESS=$(head -n1 ./neardev_ptoken/dev-account)

echo "Meta Pool: "$METAPOOL_CONTRACT_ADDRESS
echo "Katherine: "$KATHERINE_CONTRACT_ADDRESS
echo "pTOKEN: "$PTOKEN_CONTRACT_ADDRESS

KATHERINE_OWNER_ID="kate_tester3.testnet" # This account is owner of all the 3 contracts.
KICKSTARTER_OWNER_ID="kate_kickstarter_owner.testnet"
SUPPORTER_ID="kate_supporter.testnet"
YOCTO_UNITS="000000000000000000000000"
TOTAL_PREPAID_GAS=300000000000000

#echo "------------------ Registering accounts"
#NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS register_account '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID
#NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS register_account '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID
#NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $KATHERINE_CONTRACT_ADDRESS
#NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID
#NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID
#NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $KATHERINE_CONTRACT_ADDRESS

echo "------------------ Sending stNear to the supporter"
NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer '{"receiver_id": "'$SUPPORTER_ID'", "amount": "'15$YOCTO_UNITS'"}' --accountId $KATHERINE_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

echo "------------------ Checking supporter stNear balance"
NEAR_ENV=testnet near view $METAPOOL_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

echo "------------------ Sending pToken to the kickstarter"
NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS ft_transfer '{"receiver_id": "'$KICKSTARTER_OWNER_ID'", "amount": "'60$YOCTO_UNITS'"}' --accountId $KATHERINE_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

echo "------------------ Checking kickstarter pToken balance"
NEAR_ENV=testnet near view $PTOKEN_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID

# Create a Kickstarter project
KICKSTARTER_ID=0
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="The_Best_Project_Ever"
KICKSTARTER_SLUG="the-best-project-ever-${NOW_IN_MILLISECS}"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
# Cierre de período de fondeo
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 7200000))
echo "------------------ Creating a Kickstarter"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "deposits_hard_cap": "'9$YOCTO_UNITS'", "max_tokens_to_release_per_stnear": "'2$YOCTO_UNITS'"}' --accountId $KATHERINE_OWNER_ID

KICKSTARTER_ID=$(NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

# Create 2 goals
GOAL_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_END_DATE=$(($GOAL_CLIFF_DATE + 60000))
GOAL_UNFREEZE_DATE=$GOAL_END_DATE

GOAL_1_DESIRED_AMOUNT="5"$YOCTO_UNITS
GOAL_1_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
echo "------------------ Creating Goal #1"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Silver", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID

GOAL_2_DESIRED_AMOUNT="8"$YOCTO_UNITS
GOAL_2_TOKENS_TO_RELEASE="2"$YOCTO_UNITS
echo "------------------ Creating Goal #2"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Gold", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID

# FRONTEND CALL: get_active_projects
echo "------------------ FRONTEND: Get Active Projects"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_active_projects '{"from_index": 0, "limit": 10}' --accountId $KATHERINE_OWNER_ID

# Sending pTokens to Kickstarter
echo "------------------ Sending pTokens to the contract"
NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$KATHERINE_CONTRACT_ADDRESS'", "amount": "'20$YOCTO_UNITS'", "msg": "'$KICKSTARTER_ID'"}' --accountId $KICKSTARTER_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Sending stnear tokens to Kickstarter
NOW_IN_SECS=$(date +%s)
OPEN_DATE_IN_SECS=$(($KICKSTARTER_OPEN_DATE / 1000))
WAITING_SECONDS=$(($OPEN_DATE_IN_SECS - $NOW_IN_SECS))
echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
sleep $WAITING_SECONDS
echo "------------------ Sending stNEAR to the contract"
NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$KATHERINE_CONTRACT_ADDRESS'", "amount": "'$GOAL_1_DESIRED_AMOUNT'", "msg": "'$KICKSTARTER_ID'"}' --accountId $SUPPORTER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

echo "------------------ BUGS: 🪳 🐞"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_projects '{"supporter_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

echo "------------------ FRONTEND: Supporter Dashboard"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_detailed_list '{"supporter_id": "'$SUPPORTER_ID'", "st_near_price": "'$(date +%s)000000000000000'", "from_index": 0, "limit": 10}' --accountId $KATHERINE_OWNER_ID

# Evaluating project
#NOW_IN_SECS=$(date +%s)
#CLOSE_DATE_IN_SECS=$(($KICKSTARTER_CLOSE_DATE / 1000))
#WAITING_SECONDS=$(($CLOSE_DATE_IN_SECS - $NOW_IN_SECS))
#echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
#sleep $(($WAITING_SECONDS + 1))
#
## ROBOT
#echo "------------------ ROBOT: Get Projects"
#NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_kickstarters_to_process '{"from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID
#
#echo "------------------ ROBOT: Processing kickstarter"
#NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS process_kickstarter '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas 300000000000000

echo "------------------ Get project details"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_project_details '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID

echo "------------------ Checking kickstarter pToken balance"
NEAR_ENV=testnet near view $PTOKEN_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID

echo "------------------ Withdraw Kickstarter Excedent"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS kickstarter_withdraw_excedent '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KICKSTARTER_OWNER_ID --gas 300000000000000

echo "------------------ Checking kickstarter pToken balance"
NEAR_ENV=testnet near view $PTOKEN_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID

echo "------------------ BUGS: 🪳 🐞"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_projects '{"supporter_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

echo "------------------ BUGS: 🪳 🐞 🕷"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_detailed_list '{"supporter_id": "'$SUPPORTER_ID'", "st_near_price": "'$(date +%s)000000000000000'", "from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID

echo ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Second Kickstarter"

# Create a Kickstarter project
KICKSTARTER_ID=1
NOW_IN_MILLISECS=$(($(date +%s) * 1000))
KICKSTARTER_NAME="The_Second_Best_Project_Ever"
KICKSTARTER_SLUG="the-best-project-ever-${NOW_IN_MILLISECS}"
KICKSTARTER_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
KICKSTARTER_CLOSE_DATE=$(($KICKSTARTER_OPEN_DATE + 172800000))
echo "------------------ Creating a Kickstarter"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "deposits_hard_cap": "'5$YOCTO_UNITS'", "max_tokens_to_release_per_stnear": "'1$YOCTO_UNITS'"}' --accountId $KATHERINE_OWNER_ID

KICKSTARTER_ID=$(NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS get_kickstarter_id_from_slug '{"slug": "'$KICKSTARTER_SLUG'"}' --accountId $KATHERINE_OWNER_ID | grep "https://explorer.testnet.near.org/transactions/" -A 1 | grep -v "https://explorer.testnet.near.org/transactions")

# Create 2 goals
GOAL_CLIFF_DATE=$(($KICKSTARTER_CLOSE_DATE + 60000))
GOAL_END_DATE=$(($GOAL_CLIFF_DATE + 60000))
GOAL_UNFREEZE_DATE=$GOAL_END_DATE

GOAL_1_DESIRED_AMOUNT="2"$YOCTO_UNITS
GOAL_1_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
echo "------------------ Creating Goal #1"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Silver", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID

GOAL_2_DESIRED_AMOUNT="4"$YOCTO_UNITS
GOAL_2_TOKENS_TO_RELEASE="1"$YOCTO_UNITS
echo "------------------ Creating Goal #2"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "Gold", "desired_amount": "'$GOAL_2_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_UNFREEZE_DATE', "tokens_to_release_per_stnear": "'$GOAL_2_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_CLIFF_DATE', "end_timestamp": '$GOAL_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID

# FRONTEND CALL: get_active_projects
echo "------------------ FRONTEND: Get Active Projects"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_active_projects '{"from_index": 0, "limit": 10}' --accountId $KATHERINE_OWNER_ID

# Sending pTokens to Kickstarter
echo "------------------ Sending pTokens to the contract"
NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$KATHERINE_CONTRACT_ADDRESS'", "amount": "'6$YOCTO_UNITS'", "msg": "'$KICKSTARTER_ID'"}' --accountId $KICKSTARTER_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

# Sending stnear tokens to Kickstarter
NOW_IN_SECS=$(date +%s)
OPEN_DATE_IN_SECS=$(($KICKSTARTER_OPEN_DATE / 1000))
WAITING_SECONDS=$(($OPEN_DATE_IN_SECS - $NOW_IN_SECS))
echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
sleep $WAITING_SECONDS
echo "------------------ Sending stNEAR to the contract"
NEAR_ENV=testnet near call $METAPOOL_CONTRACT_ADDRESS ft_transfer_call '{"receiver_id": "'$KATHERINE_CONTRACT_ADDRESS'", "amount": "'$GOAL_1_DESIRED_AMOUNT'", "msg": "'$KICKSTARTER_ID'"}' --accountId $SUPPORTER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS

echo "------------------ BUGS: 🪳 🐞"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_projects '{"supporter_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

echo "------------------ FRONTEND: Supporter Dashboard"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_detailed_list '{"supporter_id": "'$SUPPORTER_ID'", "st_near_price": "'$(date +%s)000000000000000'", "from_index": 0, "limit": 10}' --accountId $KATHERINE_OWNER_ID

## Evaluating project
#NOW_IN_SECS=$(date +%s)
#CLOSE_DATE_IN_SECS=$(($KICKSTARTER_CLOSE_DATE / 1000))
#WAITING_SECONDS=$(($CLOSE_DATE_IN_SECS - $NOW_IN_SECS))
#echo "------------------ Waiting for "$WAITING_SECONDS" seconds!"
#sleep $(($WAITING_SECONDS + 1))
#
## ROBOT
#echo "------------------ ROBOT: Get Projects"
#NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_kickstarters_to_process '{"from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID
#
#echo "------------------ ROBOT: Processing kickstarter"
#NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS process_kickstarter '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID --gas 300000000000000
#
echo "------------------ Get project details"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_project_details '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $SUPPORTER_ID

echo "------------------ Checking kickstarter pToken balance"
NEAR_ENV=testnet near view $PTOKEN_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID

echo "------------------ Withdraw Kickstarter Excedent"
NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS kickstarter_withdraw_excedent '{"kickstarter_id": '$KICKSTARTER_ID'}' --accountId $KICKSTARTER_OWNER_ID --gas 300000000000000

echo "------------------ Checking kickstarter pToken balance"
NEAR_ENV=testnet near view $PTOKEN_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$KICKSTARTER_OWNER_ID'"}' --accountId $KICKSTARTER_OWNER_ID

echo "------------------ BUGS: 🪳 🐞"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_projects '{"supporter_id": "'$SUPPORTER_ID'"}' --accountId $SUPPORTER_ID

echo "------------------ BUGS: 🪳 🐞 🕷"
NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_supported_detailed_list '{"supporter_id": "'$SUPPORTER_ID'", "st_near_price": "'$(date +%s)000000000000000'", "from_index": 0, "limit": 10}' --accountId $SUPPORTER_ID
