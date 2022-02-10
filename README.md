# Katherine Fundraising

Allow any project to bootstrap liquidity through staking on Meta Pool.

## Build the contract

Run the `build.sh` script.

```sh
./build.sh
```

## Deploy the contract in Testnet

https://docs.near.org/docs/tools/near-cli#near-deploy

```sh
export NEAR_ENV=testnet

rm -rf neardev/ && near dev-deploy --wasmFile res/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.testnet", "staking_goal": 10}' && export $(grep -v '^#' neardev/dev-account.env | xargs)
```

A new account will be created for the contract. Note how the last command exported CONTRACT_NAME.

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