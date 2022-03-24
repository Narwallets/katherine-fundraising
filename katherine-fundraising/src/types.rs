use uint::construct_uint;

use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::{AccountId};
use near_sdk::serde::{Serialize, Deserialize};


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
    pub total_supporters: u64,
    pub open_timestamp: EpochMillis,
    pub close_timestamp: EpochMillis,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GoalJSON {
    pub id: GoalIdJSON,
    pub name: String,
    pub desired_amount: BalanceJSON,
    pub unfreeze_timestamp: EpochMillis,
    pub tokens_to_release: BalanceJSON,
    pub cliff_timestamp: EpochMillis,
    pub end_timestamp: EpochMillis,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterStatusJSON {
    pub successful: Vec<KickstarterIdJSON>,
    pub unsuccessful: Vec<KickstarterIdJSON>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterSupporterJSON {
    pub supporter_id: SupporterIdJSON,
    pub kickstarter_id: KickstarterIdJSON,
    pub total_deposited: BalanceJSON,
}
