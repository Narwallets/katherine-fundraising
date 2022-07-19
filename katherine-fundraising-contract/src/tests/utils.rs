#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use super::*;
use near_sdk::json_types::Base58PublicKey;
use near_sdk::{AccountId, MockedBlockchain, PromiseResult, VMContext};

use std::time::{SystemTime, UNIX_EPOCH};
use slug;

/// Tests constants
pub const SYSTEM_ACCOUNT: &'static str = "system";
pub const CONTRACT_ACCOUNT: &'static str = "contract";
pub const OWNER_ACCOUNT: &'static str = SYSTEM_ACCOUNT;
pub const SUPPORTER_ACCOUNT: &'static str = "owner";
pub const SUPPORTER_ID: usize = 0;
pub const STAKING_GOAL: u128 = 1000;
pub const TEST_INITIAL_BALANCE: u128 = 100;
pub const DEPOSIT_AMOUNT: u128 = 200;
pub const START_TIME_IN_DAYS: u64 = 1777;
pub const KICKSTARTER_NAME: &'static str = "test_kickstarter";
pub const KICKSTARTER_SLUG: &'static str = "test_kickstarter_slug";
pub const METAPOOL_CONTRACT_ADDRESS: &'static str = "meta-v2.pool.testnet";

pub const YOCTO_UNITS: u128 = 1_000_000_000_000_000_000_000_000;

/// Init Katherine Consts
pub fn get_min_deposit_amount() -> BalanceJSON { U128::from(1 * YOCTO_UNITS) }
pub fn get_deposits_hard_cap() -> BalanceJSON { U128::from(100 * YOCTO_UNITS) }
pub fn get_max_tokens_to_release_per_stnear() -> BalanceJSON { U128::from(27 * YOCTO_UNITS) }
pub const KATHERINE_FEE_PERCENT: BasisPoints = 200;

pub struct KickstarterGoalTimes {
    pub open_timestamp: EpochMillis,
    pub close_timestamp: EpochMillis, 
    pub unfreeze_timestamp: EpochMillis,
    pub cliff_timestamp: EpochMillis,
    pub end_timestamp: EpochMillis,
}

impl KickstarterGoalTimes {
    pub fn new(now: Now) -> Self {
        // Kickstarter parameters
        let open_timestamp = now.increment_min(1);
        let close_timestamp = open_timestamp.increment_min(1);

        // Kickstarter's Goal parameters
        let cliff_timestamp = close_timestamp.increment_min(1);
        let end_timestamp = cliff_timestamp.increment_min(1);
        let unfreeze_timestamp = end_timestamp.clone();
        Self {
            open_timestamp: open_timestamp.to_epoch_milis(),
            close_timestamp: close_timestamp.to_epoch_milis(),
            unfreeze_timestamp: unfreeze_timestamp.to_epoch_milis(),
            cliff_timestamp: cliff_timestamp.to_epoch_milis(),
            end_timestamp: end_timestamp.to_epoch_milis(),
        }
    }
}


pub struct Now {
    nanosecs: u128
}

impl Now {
    pub fn new() -> Self {
        Self {
            nanosecs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos()
        }
    }

    pub fn to_epoch_milis(&self) -> EpochMillis {
        (self.nanosecs / 1000) as EpochMillis
    }

    pub fn to_nanos(&self) -> u128 {
        self.nanosecs
    }

    pub fn increment_min(&self, min: u128) -> Now {
        Now { nanosecs: self.nanosecs + (min * 60 * 1_000_000_000) }
    }
}

impl Copy for Now {}

impl Clone for Now {
    fn clone(&self) -> Self {
        *self
    }
}


/// Get VMContext for Unit tests
pub fn get_context(
    current_account_id: AccountId,
    predecessor_account_id: AccountId,
    account_balance: u128,
    account_locked_balance: u128,
    is_view: bool,
) -> VMContext {
    VMContext {
        current_account_id,
        signer_account_id: predecessor_account_id.clone(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id,
        input: vec![],
        block_index: 1,
        block_timestamp: Now::new().to_epoch_milis(),
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
    return near_amount * 10u128.pow(24);
}

/// Convert to Timestamp
pub fn to_ts(num_days: u64) -> u64 {
    // 2018-08-01 UTC in nanoseconds
    1533081600_000_000_000 + to_nanos(num_days)
}

/// Convert days to nanoseconds
pub fn to_nanos(num_days: u64) -> u64 {
    return num_days * 86400_000_000_000;
}

pub fn get_katherine_owner() -> AccountId {
    "katherine.owner.near".to_string()
}

pub fn get_metapool_address() -> AccountId {
    "meta-v2.pool.testnet".to_string()
}

pub fn get_kickstarter_owner(id: u32) -> AccountId {
    format!("kickstarter_{}.near", id)
}

pub fn get_kickstarter_token(id: u32) -> AccountId {
    format!("kickstarter_{}.near", id)
}

pub struct TestKickstarter {
    pub name: String,
    pub slug: String,
    pub owner_id: AccountId,
    pub open_timestamp: EpochMillis,
    pub close_timestamp: EpochMillis,
    pub token_contract_address: AccountId,
    pub deposits_hard_cap: BalanceJSON,
    pub max_tokens_to_release_per_stnear: BalanceJSON,
    pub token_contract_decimals: u8,
}

impl TestKickstarter {
    pub fn new(
        id: u32,
        now_open_delta_mins: u128,
        open_close_delta_mins: u128
    ) -> Self {
        let name = format!("kickstarter_{}", id);
        let open = Now::new().increment_min(now_open_delta_mins);
        let close = open.increment_min(open_close_delta_mins);
        Self {
            name: name.clone(),
            slug: slug::slugify(&name),
            owner_id: get_kickstarter_owner(id),
            open_timestamp: open.to_epoch_milis(),
            close_timestamp: close.to_epoch_milis(),
            token_contract_address: get_kickstarter_token(1),
            deposits_hard_cap: get_deposits_hard_cap(),
            max_tokens_to_release_per_stnear: get_max_tokens_to_release_per_stnear(),
            token_contract_decimals: 24,
        }
    }
}
