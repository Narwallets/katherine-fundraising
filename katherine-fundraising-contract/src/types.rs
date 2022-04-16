use uint::construct_uint;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

pub type BalanceJSON = U128;

pub type KickstarterId = u32;
pub type KickstarterIdJSON = u32;
pub type GoalId = u8;
pub type GoalIdJSON = u8;

pub type EpochMillis = u64;
pub type BasisPoints = u32;
pub type SupporterId = AccountId;
pub type SupporterIdJSON = ValidAccountId;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterJSON {
    pub id: KickstarterIdJSON,
    pub total_supporters: u32,
    pub total_deposited: BalanceJSON,
    pub open_timestamp: EpochMillis,
    pub close_timestamp: EpochMillis,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterDetailsJSON {
    pub id: KickstarterIdJSON,
    pub total_supporters: u32,
    pub total_deposited: BalanceJSON,
    pub open_timestamp: EpochMillis,
    pub close_timestamp: EpochMillis,
    pub token_contract_address: AccountId,
    pub stnear_price_at_freeze: BalanceJSON,
    pub stnear_price_at_unfreeze: BalanceJSON,
    pub goals: Vec<GoalJSON>,
    pub active: bool,
    pub successful: Option<bool>,
    pub winner_goal_id: Option<u8>,
    pub enough_reward_tokens: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GoalJSON {
    pub id: GoalIdJSON,
    pub name: String,
    pub desired_amount: BalanceJSON,
    pub unfreeze_timestamp: EpochMillis,
    pub tokens_to_release_per_stnear: BalanceJSON,
    pub cliff_timestamp: EpochMillis,
    pub end_timestamp: EpochMillis,
    pub reward_installments: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterStatusJSON {
    pub successful: Vec<KickstarterIdJSON>,
    pub unsuccessful: Vec<KickstarterIdJSON>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ActiveKickstarterJSON {
    pub active: Vec<KickstarterJSON>,
    pub open: Vec<KickstarterJSON>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterSupporterJSON {
    pub supporter_id: SupporterIdJSON,
    pub kickstarter_id: KickstarterIdJSON,
    pub total_deposited: BalanceJSON,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SupporterDetailedJSON {
    pub kickstarter_id: KickstarterIdJSON,
    pub supporter_deposit: BalanceJSON,
    pub rewards: Option<BalanceJSON>,
    pub available_rewards: Option<BalanceJSON>,
    pub active: bool,
    pub successful: Option<bool>,
}
