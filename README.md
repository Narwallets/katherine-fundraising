# Katherine Fundraising

Allow any project to bootstrap liquidity through staking on Meta Pool.

## Project Definition

From now on, to be consistent with the naming convention in the code, we'll name the Projects as **Kickstarter**.

A Kickstarter could use the Katherine Fundraising Contract to raise funds leveraging the yield generated by staking Near tokens in Meta Pool.

The Kickstarter life cycle:

![Kickstarter Life Cycle](media/lifecycle.png)

These are the functions to interact with Katherine.

## Interact with Katherine

### 1. Create a Kickstarter

**Katherine admin**:
- [create_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#create_kickstarter)
- [update_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#update_kickstarter)

**Public**:
- [get_kickstarter_id_from_slug](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarter_id_from_slug)
- [get_total_kickstarters](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_total_kickstarters)
- [get_kickstarters](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarters)
- [get_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarter)

### 2. Create the Kickstarter Goals

**Katherine admin and Kickstarter**:
- [create_goal](https://github.com/Narwallets/katherine-fundraising/tree/dev#create_goal)
- [delete_last_goal](https://github.com/Narwallets/katherine-fundraising/tree/dev#delete_last_goal)

**Public**:
- [get_kickstarter_total_goals](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarter_total_goals)
- [get_kickstarter_goal](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarter_goal)

**Kickstarter**:
- [ft_transfer_call](https://github.com/Narwallets/katherine-fundraising/tree/dev#ft_transfer_call) (Called on Project/Token Contract)

### 3. Funding period begins

**Supporter**:
- [withdraw](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw)
- [withdraw_all](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw_all)
- [ft_transfer_call](https://github.com/Narwallets/katherine-fundraising/tree/dev#ft_transfer_call) (Called on Meta Pool)

**Public**:
- [get_active_projects](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_active_projects)
- [get_project_details](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_project_details)
- [get_supporter_total_deposit_in_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_supporter_total_deposit_in_kickstarter)

### 4. Evaluate Goal

**Robot**:
- [get_kickstarters_to_process](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarters_to_process)
- [process_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#process_kickstarter)

**Public**:
- unfreeze_kickstarter_funds

### 5. Close unsuccessful Kickstarter

**Kickstarter**:
- kickstarter_withdraw_excedent

**Supporter**:
- [withdraw](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw)
- [withdraw_all](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw_all)

### 6. Freeze Supporter funds

**Kickstarter**:
- kickstarter_withdraw_excedent

**Supporter**:
- get_supporter_estimated_stnear - When the supporter funds are freezed by the Kickstarter, use this function to calculate an estimation of the current amount of stNear that Katherine has for the supporter.

**Supporter Dashboard**:
- get_supported_projects
- get_supported_detailed_list

### 7. Allow the Kickstarter to withdraw stNear

**Kickstarter**:
- withdraw_stnear_interest

### 8. Allow the Supporter to withdraw project Tokens

**Supporter**:
- withdraw_kickstarter_tokens

**Public**:
- get_supporter_total_rewards
- get_supporter_available_rewards

### 9. Allow the Supporter to withdraw stNear

**Robot**:
- unfreeze_kickstarter_funds

**Supporter**:
- [withdraw](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw)
- [withdraw_all](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw_all)

## Function list

### **create_kickstarter**

To create a Kickstarter, Katherine admin must call:

```rust
fn create_kickstarter(
    name: String,
    slug: String,
    owner_id: String,
    open_timestamp: u64,
    close_timestamp: u64,
    token_contract_address: String,
) -> u32
```

An example using the terminal:

```sh
NEAR_ENV=testnet near call $CONTRACT_NAME create_kickstarter '{"name": "'$KICKSTARTER_NAME'", "slug": "'$KICKSTARTER_SLUG'", "owner_id": "'$KICKSTARTER_OWNER_ID'", "open_timestamp": '$KICKSTARTER_OPEN_DATE', "close_timestamp": '$KICKSTARTER_CLOSE_DATE', "token_contract_address": "'$KICKSTARTER_TOKEN_ADDRESS'"}' --accountId $KATHERINE_OWNER_ID
```

The returned value is the **Kickstarter Id**.

### **update_kickstarter**

Update the Kickstarter ONLY before the funding period opens.

```rust
fn update_kickstarter(
    id: u32,
    name: String,
    slug: String,
    owner_id: String,
    open_timestamp: u64,
    close_timestamp: u64,
    token_contract_address: String,
)
```

### **get_kickstarter_id_from_slug**

You could retreat the Kickstarter Id from the Kickstarter unique slug.

```rust
fn get_kickstarter_id_from_slug(slug: String) -> u32 
```

### **get_total_kickstarters**

Get the total number of Kickstarters, this value equals the maximum Kickstarter Id.

```rust
fn get_total_kickstarters() -> u32
```

### **get_kickstarters**

Get a list of Kickstarters starting from index `from_index`. See [get_kickstarter](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_kickstarter) to get the details of the `KickstarterJSON` response.

```rust
fn get_kickstarters(from_index: usize, limit: usize) -> Vec<KickstarterJSON>
```

### **get_kickstarter**

Get the simple information about the Kickstarter with the `KickstarterJSON` object. To get a more detailed view of the Kickstarter use [get_project_details](https://github.com/Narwallets/katherine-fundraising/tree/dev#get_project_details).

```rust
fn get_kickstarter(kickstarter_id: KickstarterIdJSON) -> KickstarterJSON
```

The `KickstarterJSON` response:

```rust
struct KickstarterJSON {
    pub id: u32,
    pub total_supporters: u32,
    pub total_deposited: String,
    pub open_timestamp: u64,
    pub close_timestamp: u64,
}
```

### **create_goal**

To create one of the multiple goals, the MAX number of goals is 5:

```rust
fn create_goal(
    kickstarter_id: u32,
    name: String,
    desired_amount: String,
    unfreeze_timestamp: u64,
    tokens_to_release: String,
    cliff_timestamp: u64,
    end_timestamp: u64,
) -> u8
```

An example using the terminal:

```sh
NEAR_ENV=testnet near call $CONTRACT_NAME create_goal '{"kickstarter_id": '$KICKSTARTER_ID', "name": "'$GOAL_1_NAME'", "desired_amount": "'$GOAL_1_DESIRED_AMOUNT'", "unfreeze_timestamp": '$GOAL_1_UNFREEZE_DATE', "tokens_to_release": "'$GOAL_1_TOKENS_TO_RELEASE'", "cliff_timestamp": '$GOAL_1_CLIFF_DATE', "end_timestamp": '$GOAL_1_END_DATE'}' --accountId $KICKSTARTER_OWNER_ID
```

The returned value is the **Goal Id**.

The Kickstarter owner could detele a goal, before the funding period is open.

### **delete_last_goal**

```rust
fn delete_last_goal(kickstarter_id: u32)
```

### **get_kickstarter_total_goals**

Returns the number of goals for a Kickstarter.

```rust
fn get_kickstarter_total_goals(kickstarter_id: u32) -> u8
```

### **get_kickstarter_goal**

Return the Goal using the Goal Id.

```rust
fn get_kickstarter_goal(
        kickstarter_id: u32,
        goal_id: u8,
    ) -> GoalJSON
```

The `GoalJSON` response:

```rust
struct GoalJSON {
    pub id: u8,
    pub name: String,
    pub desired_amount: String,
    pub unfreeze_timestamp: u64,
    pub tokens_to_release: String,
    pub cliff_timestamp: u64,
    pub end_timestamp: u64,
}
```

### **ft_transfer_call**

Funds can be transfered to Kathering using the standard transfer with callback.

When an account send **stNear** for the first time to Katherine, a supporter is created.

The `"msg"` argument MUST be included with the `Kickstarter_id`. If the `msg` does not contain a valid `Kickstarter_id` the funds will be rejected and returned to the sending account.

```rust
fn ft_transfer_call(
        receiver_id: String,    // Katherine Contract Address
        amount: String,
        msg: String,
    )
```

If the funds are being send by the Kickstarter, the **pTokens**, the tokens must be sent from the token address reported when the Kickstarter was created.

### **withdraw**

This function is for the Supporters to withdraw stNear. If it's called during the funding period, all the tokens could be withdraw. This same function works for stNear withdraw after the funds are unfreezed.

```rust
fn withdraw(amount: String, kickstarter_id: u32)
```

### **withdraw_all**

Same as [withdraw](https://github.com/Narwallets/katherine-fundraising/tree/dev#withdraw), but automatically calculate all the available tokens for the user.

```rust
fn withdraw_all(kickstarter_id: KickstarterIdJSON)
```

### **get_active_projects**

This is a function destinated for the FRONTEND to call the active and open projects.

**Active** project are waiting for funding. **Open** projects are in funding period.

```rust
fn get_active_projects(
        from_index: u32,
        limit: u32,
    ) -> Option<ActiveKickstarterJSON>
```

If the returned value is `null` then the `from_index` value is larger than the total number of kickstarters.

### **get_project_details**

Get all the details from a particular Kickstarter.

```rust
fn get_project_details(kickstarter_id: KickstarterIdJSON) -> KickstarterDetailsJSON
```

The result is the `KickstarterDetailsJSON`:

```rust
struct KickstarterDetailsJSON {
    pub id: u32,
    pub total_supporters: u32,
    pub total_deposited: String,
    pub open_timestamp: u64,
    pub close_timestamp: u64,
    pub token_contract_address: String,
    pub stnear_price_at_freeze: String,
    pub stnear_price_at_unfreeze: String,
    pub goals: Vec<GoalJSON>,
}
```

### **get_supporter_total_deposit_in_kickstarter**

An **important** function to get the total amount that a supporter has deposited in an specific Kickstarter. If the Supporter is not part of the Kickstarter then the function will `panic`.

```rust
fn get_supporter_total_deposit_in_kickstarter(
        supporter_id: String,
        kickstarter_id: u32,
    ) -> String
```

### **get_kickstarters_to_process**

This is a view function for the **robot**. It returns a list of the successful and unsuccessful `Kickstarter Id`.

```rust
pub fn get_kickstarters_to_process(
    from_index: KickstarterIdJSON,
    limit: KickstarterIdJSON,
) -> Option<KickstarterStatusJSON>
```

The result could be `null`, this would mean that the maximum number of kickstarter has reached. The structure is:

```rust
pub struct KickstarterStatusJSON {
    pub successful: Vec<KickstarterIdJSON>,
    pub unsuccessful: Vec<KickstarterIdJSON>,
}
```

### **process_kickstarter**

This is a call function for the **robot**. It processes the successful and unsuccessful Kickstarters to **evaluate** if a Goal was reached or not.

```rust
fn process_kickstarter(&mut self, kickstarter_id: KickstarterIdJSON)
```

Contract Logic:

![Katherine Contract Logic](media/logic1.png)

The Ticket system:

![Ticket System](media/logic2.png)

## Important Assumptions

- Supporters after doing a deposit to a Kickstarter, could recover the funds before they get locked.
- Goal 1 gets all the stNEAR obtained if the Goal 2 is not met.

## Contract Functions

When a user deposits to fund a project, all of their stNEAR tokens are `ready_to_fund`.

- If the project is unsuccessful, fund are moved from `ready_to_fund` to `available`.
- If the project is successful, funds are moved from `ready_to_fund` to `locked`. When the locking period ends, fund are move backed from `locked` to `available`. Note that less stNEAR will move back, however the value in NEAR will be the same.


```text
create_project() - Kickstarter
deposit_and_stake() - User

user_withdrawa() - User
get_back_rewards() - Kickstarter
```


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

rm -rf neardev/ && near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": "jomsox.testnet", "staking_goal": 10}' && export $(grep -v '^#' neardev/dev-account.env | xargs)
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
To install the kurtosis CLI follow the installation documentation: https://docs.kurtosistech.com/installation.html

1. Build the contract

```sh
RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
```

2. Deploy the contract in a local node

https://docs.near.org/docs/tools/near-cli#near-deploy

```sh
cd katherine-fundraising-contract

local_near deploy --accountId jomsox.test.near --wasmFile target/wasm32-unknown-unknown/release/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": "jomsox.test.near", "staking_goal": 10000}'
```

3. Deposit to the contract

https://docs.near.org/docs/tools/near-cli#near-call

```sh
local_near call jomsox.test.near deposit_and_stake --accountId jomsox.test.near --deposit 2
```

# References

* https://github.com/Narwallets/meta-pool
* https://docs.near.org/docs/tools/kurtosis-localnet
* https://docs.kurtosistech.com/installation.html