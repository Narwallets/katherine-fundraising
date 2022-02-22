use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::{log, Promise, env, near_bindgen, AccountId, PanicOnDefault, Balance, Gas};

pub mod supporter;
pub use crate::supporter::*;

pub mod kickstarter;
pub use crate::kickstarter::*;

pub mod goal;
pub use crate::goal::*;

pub mod ticket;
pub use crate::ticket::*;

pub mod types;
pub use crate::types::*;

pub mod utils;
pub use crate::utils::*;

mod internal;
mod metapool;

const GAS: Gas = 20_000_000_000_000;
// const METAPOOL_CONTRACT_ADDRESS: AccountId = String::from("meta-v2.pool.testnet");


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {

    pub owner_id: AccountId,

    pub supporters: UnorderedMap<AccountId, Supporter>,

    pub kickstarters: UnorderedMap<KickstarterId, Kickstarter>,

    pub total_available: Balance,

    pub staking_goal: Balance,
    
    /// min amount accepted as deposit or stake
    pub min_deposit_amount: Balance,

    pub metapool_contract_address: AccountId,
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(owner_id: AccountId, staking_goal: Balance) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(b"A".to_vec()),
            kickstarters: UnorderedMap::new(b"A".to_vec()),
            total_available: 0,
            staking_goal: staking_goal * NEAR,
            min_deposit_amount: 1 * NEAR,
            metapool_contract_address: String::from("meta-v2.pool.testnet"),
        }
    }

    #[payable]
    pub fn deposit_and_stake(&mut self, amount: Balance) {
        let supporter: AccountId = env::predecessor_account_id();
        // let supporter_stnear: Promise = self.take_supporter_stnear(supporter, amount);
        self.internal_deposit(amount);
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: Balance) -> Promise {
        self.internal_withdraw(amount)
    }
    /// Withdraws ALL from from "UNSTAKED" balance *TO MIMIC core-contracts/staking-pool .- core-contracts/staking-pool only has "unstaked" to withdraw from
    pub fn withdraw_all(&mut self) -> Promise {
        let supporter = self.internal_get_supporter(&env::predecessor_account_id());
        self.internal_withdraw(supporter.available)
    }

    pub fn evaluate_at_due(&mut self) {
        let current_timestamp = env::block_timestamp();
        for (kickstarter_id, kickstarter) in self.kickstarters.to_vec().iter() {
            if kickstarter.active && kickstarter.finish_timestamp < current_timestamp {
                let mut kickstarter = self.internal_get_kickstarter(&kickstarter_id);
                if self.internal_evaluate_goals(&kickstarter) {
                    log!("The project {} with id: {} was successful!", kickstarter.name, kickstarter_id);
                    kickstarter.active = false;
                    kickstarter.succesful = true;
                    self.internal_locking_supporters_funds(&kickstarter)
                } else {
                    log!("The project {} with id: {} was unsuccessful!", kickstarter.name, kickstarter_id);
                    kickstarter.active = false;
                    kickstarter.succesful = false;
                    self.internal_freeing_supporters_funds(&kickstarter)
                }
            }
        }

        // if self.total_available < self.staking_goal {
        //     for (supporter_id, _) in self.supporters.to_vec().iter() {
        //         let mut supporter = self.internal_get_supporter(&supporter_id);
        //         // self.transfer_back_to_account(account_id, &mut account)
        //     }
        // } else {
        //     unimplemented!()
        //     // self.internal_stake_funds()
        // }
    }

    /*****************************/
    /* staking-pool View methods */
    /*****************************/

    pub fn get_supporter_available_balance(&self, supporter_id: AccountId) -> U128String {
        let supporter = self.internal_get_supporter(&supporter_id);
        supporter.available.into()
    }

    pub fn get_contract_total_available(&self) -> U128String {
        self.total_available.into()
    }
}