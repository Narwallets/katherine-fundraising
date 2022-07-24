// use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use super::*;
use near_sdk::json_types::U128;
mod utils;
use utils::*;

use near_sdk::{testing_env, MockedBlockchain, VMContext, PromiseOrValue};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

fn new_contract() -> KatherineFundraising {
    KatherineFundraising::new(
        get_katherine_owner(),
        get_min_deposit_amount(),
        get_metapool_address(),
        KATHERINE_FEE_PERCENT
    )
}

fn get_contract_setup(context: VMContext) -> KatherineFundraising {
    testing_env!(context.clone());
    let contract = new_contract();
    contract
}

fn create_kickstarter_with_2_goals(contract: &mut KatherineFundraising, now: Now) -> KickstarterId {
    let next_kickstarter_id = contract.get_total_kickstarters();

    let kickstarter = TestKickstarter::new(next_kickstarter_id, 1, 10, now);
    let kickstarter_id = contract.create_kickstarter(
        kickstarter.name.clone(),
        kickstarter.slug.clone(),
        kickstarter.owner_id.clone(),
        kickstarter.open_timestamp,
        kickstarter.close_timestamp,
        kickstarter.token_contract_address.clone(),
        kickstarter.deposits_hard_cap,
        kickstarter.max_tokens_to_release_per_stnear,
        kickstarter.token_contract_decimals
    );

    let goal_1 = TestGoal::new(kickstarter.clone(), 1, 10, 5);
    let goal_2 = TestGoal::new(kickstarter.clone(), 2, 10, 5);

    let id = contract.create_goal(
        goal_1.kickstarter_id,
        goal_1.name,
        goal_1.desired_amount,
        goal_1.unfreeze_timestamp,
        goal_1.tokens_to_release_per_stnear,
        goal_1.cliff_timestamp,
        goal_1.end_timestamp
    );

    assert_eq!(1, contract.get_kickstarter_total_goals(kickstarter_id));
    assert_eq!(0, id, "Id of the first Goal should be 0.");

    let id = contract.create_goal(
        goal_2.kickstarter_id,
        goal_2.name,
        goal_2.desired_amount,
        goal_2.unfreeze_timestamp,
        goal_2.tokens_to_release_per_stnear,
        goal_2.cliff_timestamp,
        goal_2.end_timestamp
    );

    assert_eq!(2, contract.get_kickstarter_total_goals(kickstarter_id));
    assert_eq!(1, id, "Id of the second Goal should be 1.");

    kickstarter_id
}

#[test]
fn test_create_kickstarter_with_goals() {
    let now = Now::new();
    let context = get_context(
        get_katherine_owner(),
        get_katherine_owner(),
        utils::ntoy(100),
        0,
        false,
    );

    let mut contract = get_contract_setup(context);
    let kickstarter_id = create_kickstarter_with_2_goals(&mut contract, now);
    assert_eq!(1, contract.get_total_kickstarters());
    assert_eq!(0, kickstarter_id, "Id of the first Kickstarter should be 0.");

    let kickstarter_id = create_kickstarter_with_2_goals(&mut contract, now);
    assert_eq!(2, contract.get_total_kickstarters());
    assert_eq!(1, kickstarter_id, "Id of the first Kickstarter should be 0.");

    let kickstarters = contract.get_kickstarters(0, 10);
    assert_eq!(2, kickstarters.len());
}

fn get_ready_to_funded_kickstarter_contract(now: Now) -> KatherineFundraising {
    let context = get_timestamp_context(
        get_katherine_owner(),
        get_katherine_owner(),
        utils::ntoy(100),
        0,
        now,
    );

    let mut contract = get_contract_setup(context);
    let _ = create_kickstarter_with_2_goals(&mut contract, now);

    let context = get_timestamp_context(
        get_supporter_account(),
        get_metapool_address(),
        utils::ntoy(100),
        0,
        now.increment_min(5),
    );
    testing_env!(context.clone());
    contract
}

fn deposit_kickstarter_tokens(
    contract: &mut KatherineFundraising,
    predecessor_account_id: AccountId,
    id: KickstarterId,
    now: Now
) {
    let context = get_timestamp_context(
        get_kickstarter_owner(id),
        predecessor_account_id,
        utils::ntoy(100),
        0,
        now.increment_min(5),
    );
    testing_env!(context.clone());

    // Calculate required tokens
    let kickstarter = contract.kickstarters
        .get(id as u64)
        .expect("Unknown KickstarterId");

    let max_tokens_to_release = contract.calculate_max_tokens_to_release(&kickstarter);
    let min_tokens_to_allow_support = max_tokens_to_release
        + contract.calculate_katherine_fee(max_tokens_to_release);
    let amount = kickstarter.yocto_to_less_decimals(min_tokens_to_allow_support);

    let _ = contract.ft_on_transfer(
        get_kickstarter_owner(id).try_into().unwrap(),
        BalanceJSON::from(amount),
        format!("{}", id),
    );
}

#[test]
#[should_panic(expected="Supporters cannot deposit until the Kickstarter covers the required rewards!")]
fn test_fail_deposit_no_kickstarter_tokens() {
    let now = Now::new();
    let mut contract = get_ready_to_funded_kickstarter_contract(now.clone());
    let kickstarter_id = "0".to_string();

    let _ = contract.ft_on_transfer(
        get_supporter_account().try_into().unwrap(),
        get_supporter_deposit(),
        kickstarter_id
    );
}

#[test]
#[should_panic(expected="Not within the funding period.")]
fn test_fail_deposit_after_close() {
    let now = Now::new();
    let mut contract = get_ready_to_funded_kickstarter_contract(now.clone());
    let kickstarter_id = "0".to_string();
    let sender_id: ValidAccountId = get_supporter_account().try_into().unwrap(); // casting AccountId to ValidAccountId

    let context = get_timestamp_context(
        get_kickstarter_owner(kickstarter_id.parse().expect("Wanted a number.")),
        get_metapool_address(),
        utils::ntoy(100),
        0,
        now.increment_min(12),
    );
    testing_env!(context.clone());

    let _ = contract.ft_on_transfer(
        sender_id,
        get_supporter_deposit(),
        kickstarter_id
    );
}

#[test]
#[should_panic(expected="Deposited tokens do not correspond to the Kickstarter contract.")]
fn test_fail_deposit_bad_ptoken_contract() {
    let now = Now::new();
    let context = get_timestamp_context(
        get_katherine_owner(),
        get_katherine_owner(),
        utils::ntoy(100),
        0,
        now,
    );

    let mut contract = get_contract_setup(context);
    let kickstarter_id = create_kickstarter_with_2_goals(&mut contract, now);

    // Send the address of the kickstarter pToken.
    deposit_kickstarter_tokens(
        &mut contract,
        get_kickstarter_token(177), // Incorrect address!
        kickstarter_id,
        now
    );
}

fn get_kickstarter_one_supporter_contract(
    kickstarter_id: KickstarterId,
    deposit_amount: BalanceJSON,
    now: Now
) -> KatherineFundraising {
    let context = get_timestamp_context(
        get_katherine_owner(),
        get_katherine_owner(),
        utils::ntoy(100),
        0,
        now,
    );

    let mut contract = get_contract_setup(context);
    let kickstarter_id = create_kickstarter_with_2_goals(&mut contract, now);

    // Send the address of the kickstarter pToken.
    deposit_kickstarter_tokens(
        &mut contract,
        get_kickstarter_token(kickstarter_id),
        kickstarter_id,
        now
    );

    let context = get_timestamp_context(
        get_supporter_account(),
        get_metapool_address(),
        utils::ntoy(100),
        0,
        now.increment_min(5),
    );
    testing_env!(context.clone());

    let returned_amount = contract.ft_on_transfer(
        get_supporter_account().try_into().unwrap(),
        deposit_amount,
        format!("{}", kickstarter_id)
    );

    if let PromiseOrValue::Value(result) = returned_amount {
        assert_eq!(result, U128::from(0));
    } else {
        panic!("Error");
    }
    contract
}

#[test]
fn test_create_supporter() {
    let now = Now::new();
    let kickstarter_id = 0_u32;

    // Test with different desired amounts
    let desired_amount = get_desired_amount_from_goal_id(kickstarter_id);
    let _ = get_kickstarter_one_supporter_contract(
        kickstarter_id,
        desired_amount,
        now
    );
}

// #[test]
// fn test_evaluate_kickstarter() {
//     unimplemented!()
// }