use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

pub mod account;
pub use crate::account::*;

pub mod types;
pub use crate::types::*;

mod internal;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {

    pub owner_id: AccountId,

    pub accounts: UnorderedMap<AccountId, Account>,

    pub total_available: u128,

    pub staking_goal: u128,
    
    /// min amount accepted as deposit or stake
    pub min_deposit_amount: u128,
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(owner_id: AccountId, staking_goal: u128) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id: owner_id,
            accounts: UnorderedMap::new(b"A".to_vec()),
            total_available: 0,
            staking_goal: staking_goal,
            min_deposit_amount: 1 * NEAR,
        }
    }

    #[payable]
    pub fn deposit_and_stake(&mut self) {
        self.internal_deposit();
    }

    /*****************************/
    /* staking-pool View methods */
    /*****************************/

    pub fn get_account_available_balance(&self, account_id: AccountId) -> U128String {
        let acc = self.internal_get_account(&account_id);
        acc.available.into()
    }

    pub fn get_contract_total_available(&self) -> U128String {
        self.total_available.into()
    }
}