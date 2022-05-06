#!/bin/bash
set -e

source meta_yield.conf

if [[ "${1}" != "" ]]; then
    CONFIGURATION_FILE=$1
else
    echo "Missing configuration file"
fi

if [[ "${2}" != "" ]]; then
    PROJECT_ID=$1
else
    echo "Missing project id"
fi

# Sending Project Tokens to Project
echo "Sending project tokens: ${PROJECT_TOKEN_ADDRESS}  to the project ${PROJECT_ID}"
NEAR_ENV=testnet near call $PROJECT_TOKEN_ADDRESS ft_transfer_call '{"receiver_id": "'$CONTRACT_NAME'", "amount": "'1200000$YOCTO_UNITS'", "msg": "'$PROJECT_ID'"}' --accountId $PROJECT_OWNER_ID --depositYocto 1 --gas $TOTAL_PREPAID_GAS



