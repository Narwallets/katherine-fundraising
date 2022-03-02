use std::thread::AccessError;

use near_sdk::json_types::{U128, U64};
use near_sdk::{AccountId};

use near_sdk::serde::{Serialize, Deserialize};

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;

pub type KickstarterId = u64;
pub type KickstarterIdJSON = U64;
pub type IOUNoteId = u64;
pub type SupporterId = AccountId;


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterJSON {
    pub kickstarter_id: KickstarterIdJSON,
}