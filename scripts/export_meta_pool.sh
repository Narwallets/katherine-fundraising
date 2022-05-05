#!/bin/bash
set -e

export $(grep -v '^#' neardev_metapool/dev-account.env | xargs)
echo '{"owner_id": "'$NEAR_ACCOUNT'", "min_deposit_amount": "1000000000000000000000000", "metapool_contract_address": "'$CONTRACT_NAME'", "katherine_fee_percent": 100 }'
NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "min_deposit_amount": "1000000000000000000000000", "metapool_contract_address": "'$CONTRACT_NAME'", "katherine_fee_percent": 100 }'