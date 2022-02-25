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
    /// Incremental Folio
    pub id: IOUNoteId,

    /// Total amount of the denominated token
    pub amount: Balance,
    pub denomination: IOUNoteDenomination,

    pub supporter_id: AccountId,
    pub kickstarter_id: KickstarterId,

    /// Date when the token release starts
    pub cliff_timestamp: Timestamp,

    /// The period from cliff to end timestamp is when the user could withdraw tokens
    /// following a linear release
    pub end_timestamp: Timestamp,
}
