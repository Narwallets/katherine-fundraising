# Katherine Fundraising

Allow any project to bootstrap liquidity through staking on Meta Pool.

Contract Logic:

![Katherine Contract Logic](media/logic.png)

## Build the contract

Run the `build.sh` script.

```sh
make build
```

## Deploy the contract in Testnet

https://docs.near.org/docs/tools/near-cli#near-deploy

### Initialize contract

```sh
NEAR_ACCOUNT="imcsk8.testnet" make publish-dev-init
```

```sh
export NEAR_ENV=testnet

rm -rf neardev/ && near dev-deploy --wasmFile res/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.testnet", "staking_goal": 10}' && export $(grep -v '^#' neardev/dev-account.env | xargs)
```

A new account will be created for the contract. Note how the last command exported CONTRACT_NAME.

### Upload changes to the contract
```sh
NEAR_ACCOUNT="imcsk8.testnet" make publish-dev
```

## Deposit to the contract

https://docs.near.org/docs/tools/near-cli#near-call

Deposit to the testnet contract using a test multiple test accounts:

```sh
near call $CONTRACT_NAME deposit_and_stake --accountId jomsox.testnet --deposit 2
near call $CONTRACT_NAME deposit_and_stake --accountId huxley.testnet --deposit 11 
```

## View the contract total available amount

```sh
near view $CONTRACT_NAME get_contract_total_available '{}'
```


near call $CONTRACT_NAME evaluate_at_due --accountId huxley.testnet 

## Kickstarters

### Create kickstarter

```sh
near call dev-1645463337931-68562022007060 create_kickstarter '{"name": "test kickstarter 2", "slug": "test-kickstarter2", "finish_timestamp": 0, "open_timestamp": 0, "close_timestamp": 0, "vesting_timestamp": 0, "cliff_timestamp": 0}'  --accountId imcsk8.testnet 
```

### List kickstarters

```sh
near view dev-1645463427632-85695251757474 list_kickstarters --accountId imcsk8.testnet

```

## Deploy local Node

The `local_near` command is part of the Kurtosis development environment: https://docs.near.org/docs/tools/kurtosis-localnet.

1. Build the contract

```sh
RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
```

2. Deploy the contract in a local node

https://docs.near.org/docs/tools/near-cli#near-deploy

```sh
cd katherine-fundraising

local_near deploy --accountId jomsox.test.near --wasmFile target/wasm32-unknown-unknown/release/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.test.near", "staking_goal": 10000}'
```

3. Deposit to the contract

https://docs.near.org/docs/tools/near-cli#near-call

```sh
local_near call jomsox.test.near deposit_and_stake --accountId jomsox.test.near --deposit 2
```
