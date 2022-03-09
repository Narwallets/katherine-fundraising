use uint::construct_uint;

use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::{AccountId};

use near_sdk::serde::{Serialize, Deserialize};

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;
pub const BASIS_POINTS: u128 = 10_000;

pub const NO_DEPOSIT: u128 = 0;

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;
pub type U64String = U64;

pub type BalanceJSON = U128;

pub type KickstarterId = u32;
pub type KickstarterIdJSON = u32;

pub type IOUNoteId = u64;

pub type SupporterId = AccountId;
pub type SupporterIdJSON = ValidAccountId;

// Double Index used for the IOU Notes
// Concatenate KickstarterId + SupporterId as String.
pub type KickstarterSupporterDx = String;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterJSON {
    pub id: KickstarterIdJSON,
    pub total_supporters: U64String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterSupporterJSON {
    pub supporter_id: SupporterIdJSON,
    pub kickstarter_id: KickstarterIdJSON,
    pub total_deposited: BalanceJSON,
}

/// Struct returned from get_contract_state
/// Represents contact state as as JSON compatible struct
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GetContractStateResult {
    pub env_epoch_height: U64,
    pub contract_account_balance: U128String,
    pub total_available: U128String,
    pub total_for_staking: U128String,
    pub total_actually_staked: U128String,
    pub epoch_stake_orders: U128String,
    pub epoch_unstake_orders: U128String,
    pub total_unstaked_and_waiting: U128String,
    pub total_stake_shares: U128String,

    // How much NEAR 1 stNEAR represents, normally>1
    pub st_near_price: U128String,
    pub total_unstake_claims: U128String,
    pub reserve_for_unstake_claims: U128String,
    pub total_meta: U128String,
    pub accumulated_staked_rewards: U128String,
    pub nslp_liquidity: U128String,
    pub nslp_target: U128String,
    pub nslp_stnear_balance: U128String,
    pub nslp_share_price: U128String,
    pub nslp_total_shares: U128String,
    pub nslp_current_discount_basis_points: u16,
    pub nslp_min_discount_basis_points: u16,
    pub nslp_max_discount_basis_points: u16,
    pub accounts_count: U64,
    pub staking_pools_count: u16,
    pub min_deposit_amount: U128String,
    pub est_meta_rewards_stakers: U128String,
    pub est_meta_rewards_lp: U128String,
    pub est_meta_rewards_lu: U128String,
    pub max_meta_rewards_stakers: U128String,
    pub max_meta_rewards_lp: U128String,
    pub max_meta_rewards_lu: U128String,
}