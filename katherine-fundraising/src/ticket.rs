use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault, AccountId, Balance};
use near_sdk::serde::{Serialize, Deserialize};

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Ticket {
    pub supporter_id: AccountId,
    pub stnear_amount: Balance,
    pub spot_near_value: Balance,
}
