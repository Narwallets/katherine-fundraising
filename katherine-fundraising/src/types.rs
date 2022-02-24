use std::thread::AccessError;

use near_sdk::json_types::{U128};
use near_sdk::{AccountId};

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;

pub type KickstarterId = u64;
pub type IOUNoteId = u64;
pub type SupporterId = AccountId;