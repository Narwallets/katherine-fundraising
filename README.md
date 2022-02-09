# Katherine Fundraising

Allow any project to bootstrap liquidity through staking on Meta Pool.

## Build the contract

```sh
RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
```

## Deploy the contract in a local node

https://docs.near.org/docs/tools/near-cli#near-deploy

```sh
cd katherine-fundraising

local_near deploy --accountId jomsox.test.near --wasmFile target/wasm32-unknown-unknown/release/katherine_fundraising.wasm --initFunction new --initArgs '{"owner_id": "jomsox.test.near", "staking_goal": 10000}'
```

## Deposit to the contract

https://docs.near.org/docs/tools/near-cli#near-call

```sh
local_near call jomsox.test.near deposit_and_stake --accountId jomsox.test.near --deposit 2
```