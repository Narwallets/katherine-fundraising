use std::thread::AccessError;

use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::{AccountId};

use near_sdk::serde::{Serialize, Deserialize};

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;
pub type U64String = U64;

pub type BalanceJSON = U128;

pub type KickstarterId = u64;
pub type KickstarterIdJSON = U64;

pub type IOUNoteId = u64;

pub type SupporterId = AccountId;
pub type SupporterIdJSON = ValidAccountId;

// Double Index used for the IOU Notes
// Concatenate KickstarterId + SupporterId as String.
pub type KickstarterSupporterDx = String;


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