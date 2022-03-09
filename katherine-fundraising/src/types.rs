use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::{AccountId};
use near_sdk::serde::{Serialize, Deserialize};

pub type BalanceJSON = U128;

pub type KickstarterId = u64;
pub type KickstarterIdJSON = U64;

pub type IOUNoteId = u64;
pub type EpochMillis = u64;
pub type SupporterId = AccountId;
pub type SupporterIdJSON = ValidAccountId;


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KickstarterJSON {
    pub id: KickstarterIdJSON,
    pub total_supporters: U64,
}

pub struct KickstarterStatusJSON {
    pub status: KickstarterResult,
    pub ids: Vec<KickstarterIdJSON>
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum KickstarterResult{
    Successful,
    Unsuccessfull
}