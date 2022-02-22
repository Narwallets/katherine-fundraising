use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Funder {
    pub owner: AccountId,
    pub amount: u128,
}

/// Funder
/// TODO...
impl Default for Funder {
    fn default() -> Self {
        Self {
            owner: env::predecessor_account_id(),
            amount: 0,
        }
    }
}


/// TODO:
impl Funder {
}
