# Katherine Fundraising

Allow any project to bootstrap liquidity through staking on Meta Pool.

## Build the contract

```sh
RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
```

You could run the `build.sh` script. 

## Deploy the contract in a local node

https://docs.near.org/docs/tools/near-cli#near-deploy

```sh
cd katherine-fundraising

local_near deploy --accountId jomsox.test.near --wasmFile target/wasm32-unknown-unknown/release/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.test.near", "staking_goal": 10000}'
```

To deploy the contract in Testnet:

```sh
export NEAR_ENV=testnet


near dev-deploy --wasmFile res/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.testnet", "staking_goal": 10000}'
```

A new account will be created for the contract, export the contract address:

```sh
export KATHERINE_CONTRACT=dev-1644509922084-81337611106997
```

## Deposit to the contract

https://docs.near.org/docs/tools/near-cli#near-call

```sh
local_near call jomsox.test.near deposit_and_stake --accountId jomsox.test.near --deposit 2
```

Deposit to the testnet contract using a test multiple test accounts:

near call $KATHERINE_CONTRACT deposit_and_stake --accountId jomsox.testnet --deposit 2
near call $KATHERINE_CONTRACT deposit_and_stake --accountId huxley.testnet --deposit 36 


near view $KATHERINE_CONTRACT get_contract_total_available '{}'


near call $KATHERINE_CONTRACT evaluate_at_due --accountId huxley.testnet 