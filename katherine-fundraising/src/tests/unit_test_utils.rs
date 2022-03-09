#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use near_sdk::json_types::Base58PublicKey;
use near_sdk::{AccountId, MockedBlockchain, PromiseResult, VMContext};

use super::*;

/// Tests constants
pub const SYSTEM_ACCOUNT: &'static str = "system";
pub const CONTRACT_ACCOUNT: &'static str = "contract";
pub const OWNER_ACCOUNT: &'static str = "owner";
pub const SUPPORTER_ACCOUNT: &'static str = "owner";
pub const SUPPORTER_ID: usize = 0;
pub const STAKING_GOAL: u128 = 1000;
pub const TEST_INITIAL_BALANCE: u128 = 100;
pub const DEPOSIT_AMOUNT: u128 = 100;
pub const GENESIS_TIME_IN_DAYS: u64 = 500;
pub const KICKSTARTER_NAME: &'static str = "test_kickstarter";
pub const KICKSTARTER_SLUG: &'static str = "test_kickstarter_slug";

/// Get VMContext for Unit tests
pub fn get_context(
    predecessor_account_id: AccountId,
    account_balance: u128,
    account_locked_balance: u128,
    block_timestamp: u64,
    is_view: bool,
) -> VMContext {
    VMContext {
        current_account_id: CONTRACT_ACCOUNT.into(),
        signer_account_id: predecessor_account_id.clone(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id,
        input: vec![],
        block_index: 1,
        block_timestamp,
        epoch_height: 1,
        account_balance,
        account_locked_balance,
        storage_usage: 10u64.pow(6),
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(15),
        random_seed: vec![0, 1, 2],
        is_view,
        output_data_receivers: vec![],
    }
}

/// Convert near to yocto
pub fn ntoy(near_amount: u128) -> u128 {
    return near_amount * 10u128.pow(24)
}

/// Convert to Timestamp
pub fn to_ts(num_days: u64) -> u64 {
    // 2018-08-01 UTC in nanoseconds
    1533081600_000_000_000 + to_nanos(num_days)
}

/// Convert days to nanoseconds
pub fn to_nanos(num_days: u64) -> u64 {
    return num_days * 86400_000_000_000
}

pub fn _new_kickstarter(_context: VMContext, contract: &mut KatherineFundraising) {
	contract.create_kickstarter(
       	KICKSTARTER_NAME.into(),
       	KICKSTARTER_SLUG.into(),
       	OWNER_ACCOUNT.into(),
		// TODO: check this dates
       	to_ts(GENESIS_TIME_IN_DAYS * 60), // finish_timestamp
       	to_ts(GENESIS_TIME_IN_DAYS),      // open_timestamp
       	to_ts(GENESIS_TIME_IN_DAYS * 50), // vesting_timestamp
       	to_ts(GENESIS_TIME_IN_DAYS * 40), // close_timestamp
       	to_ts(GENESIS_TIME_IN_DAYS * 30), // cliff_timestamp
       	CONTRACT_ACCOUNT.into(),          // token_contract_address
	)
}

