use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault, AccountId, Balance};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Clone)]
pub struct Ticket {
    pub supporter_id: AccountId,
    pub stnear_amount: Balance,
    pub spot_near_value: Balance,
}