use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault, AccountId, Balance, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum IOUNoteDenomination {
    NEAR,
    KickstarterToken(String),
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct IOUNote {
    pub id: IOUNoteId,
    pub amount: Balance,
    pub denomination: IOUNoteDenomination,
    pub supporter_id: AccountId,
    pub kickstarter_id: KickstarterId,
    pub cliff_timestamp: Timestamp,
    pub end_timestamp: Timestamp,
}
