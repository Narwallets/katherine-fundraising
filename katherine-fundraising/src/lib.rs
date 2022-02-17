use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::{Promise, env, near_bindgen, AccountId, PanicOnDefault, Balance, Gas};

pub mod account;
pub use crate::account::*;

pub mod types;
pub use crate::types::*;

pub mod utils;
pub use crate::utils::*;

mod internal;

const GAS: Gas = 20_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {

    pub owner_id: AccountId,

    pub accounts: UnorderedMap<AccountId, Account>,

    pub total_available: Balance,

    pub staking_goal: Balance,
    
    /// min amount accepted as deposit or stake
    pub min_deposit_amount: Balance,
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(owner_id: AccountId, staking_goal: Balance) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            accounts: UnorderedMap::new(b"A".to_vec()),
            total_available: 0,
            staking_goal: staking_goal * NEAR,
            min_deposit_amount: 1 * NEAR,
        }
    }

    #[payable]
    pub fn deposit_and_stake(&mut self) {
        self.internal_deposit();
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: Balance) -> Promise {
        self.internal_withdraw(amount)
    }
    /// Withdraws ALL from from "UNSTAKED" balance *TO MIMIC core-contracts/staking-pool .- core-contracts/staking-pool only has "unstaked" to withdraw from
    pub fn withdraw_all(&mut self) -> Promise {
        let account = self.internal_get_account(&env::predecessor_account_id());
        self.internal_withdraw(account.available)
    }

    /// RELOCATE! Only the owner can call this function, after the due date has passed.
    pub fn evaluate_at_due(&mut self) {
        if self.total_available < self.staking_goal {
            for (account_id, _) in self.accounts.to_vec().iter() {
                let mut account = self.internal_get_account(&account_id);
                self.transfer_back_to_account(account_id, &mut account)
            }
        } else {
            self.internal_stake_funds()
        }
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