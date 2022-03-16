use uint::construct_uint;

use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::{AccountId};
use near_sdk::serde::{Serialize, Deserialize};

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;
pub type U64String = U64;

pub type BalanceJSON = U128;

pub type KickstarterId = u32;
pub type KickstarterIdJSON = u32;
pub type GoalId = u8;

pub type Milliseconds = u64;
pub type BasisPoints = u32;
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
    pub total_supporters: u32,
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
