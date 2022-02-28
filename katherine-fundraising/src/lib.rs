use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Balance, Gas, Timestamp, Promise};

pub mod supporter;
pub use crate::supporter::*;

pub mod kickstarter;
pub use crate::kickstarter::*;

pub mod iou_note;
pub use crate::iou_note::*;

pub mod goal;
pub use crate::goal::*;

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

    pub iou_notes: Vector<IOUNote>,

    /// Kickstarter list
    pub kickstarters: Vector<Kickstarter>,

    pub total_available: Balance,

    /// min amount accepted as deposit or stake
    pub min_deposit_amount: Balance,

    pub metapool_contract_address: AccountId,

    // Katherine fee is a % of the Kickstarter Token rewards.
    pub katherine_fee_percent: f32, // TODO: How should we handle this?
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(owner_id: AccountId, staking_goal: Balance) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(b"A".to_vec()),
            iou_notes: Vector::new(b"Note".to_vec()),
            kickstarters: Vector::new(b"Kickstarters".to_vec()),
            total_available: 0,
            min_deposit_amount: 1 * NEAR,
            metapool_contract_address: String::from("meta-v2.pool.testnet"),
            katherine_fee_percent: 0.1
        }
    }

    #[payable]
    pub fn deposit_and_stake(&mut self, amount: Balance) {
        unimplemented!();
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

    pub fn heartbeat(&mut self) {
        /*  
            UPDATE what the heartbeat does!
            Katherine's heartbeat 💓 must run every day:
                - Update the $NEAR / $stNEAR ratio, getting the value from Meta Pool.
                - Check if the funding period of a Kickstarter ends and evaluate the goals:
                    - If goals are met, project is successful and the funds are locked.
                    - If project is unsuccessful, funds are immediately freed to the supporters.
        */
        self.internal_evaluate_at_due();
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

    /*****************************/
    /*   Kickstarter functions   */
    /*****************************/

    /// Creates a new kickstarter entry in persistent storage
    pub fn create_kickstarter(&mut self, 
        name: String,
        slug: String,
        owner_id: AccountId,
        finish_timestamp: Timestamp,
        open_timestamp: Timestamp,
        close_timestamp: Timestamp,
        vesting_timestamp: Timestamp,
        cliff_timestamp: Timestamp,
        token_contract_address: AccountId,
    ) {
        let kickstarter = Kickstarter {
            id: self.kickstarters.len(),
            name,
            slug,
            goals: Vector::new(b"Goal".to_vec()),
            winner_goal_id: None,
            katherine_fee: None,
            supporters: Vec::new(),
            deposits: UnorderedMap::new(b"A".to_vec()),
            owner_id,
            active: true,
            successful: false,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_tokens: 0,
            locked_tokens: 0,
        };

        self.kickstarters.push(&kickstarter);
    }

    /// Returns a list of the kickstarter entries
    pub fn get_kickstarters(&self) -> Vec<Kickstarter> {
        self.kickstarters.to_vec()
    }

    pub fn delete_kickstarter(&mut self, id: KickstarterId) {
        panic!("Kickstarter must not be deleted!");
        self.kickstarters.swap_remove(id);
    }

    /// Update a kickstarter
    pub fn update_kickstarter(&mut self, 
        id: KickstarterId,
        name: String,
        slug: String,
        owner_id: AccountId,
        finish_timestamp: Timestamp,
        open_timestamp: Timestamp,
        close_timestamp: Timestamp,
        vesting_timestamp: Timestamp,
        cliff_timestamp: Timestamp,
        token_contract_address: AccountId,
    ) {
        let old_kickstarter = self.kickstarters.get(id).expect("Kickstarter Id not found!");
        
        assert!(
            old_kickstarter.open_timestamp <= env::block_timestamp(),
            "Changes are not allow after the funding period started!"
        );
        assert!(
            self.owner_id != env::predecessor_account_id(),
            "Only Katherine owner is allowed to modify the project!"
        );

        let kickstarter = Kickstarter {
            id,
            name,
            slug,
            goals: Vector::new(b"Goal".to_vec()),
            winner_goal_id: None,
            katherine_fee: None,
            supporters: Vec::new(),
            deposits: UnorderedMap::new(b"A".to_vec()),
            owner_id,
            active: true,
            successful: false,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_tokens: 0,
            locked_tokens: 0,
        };

        self.kickstarters.replace(id, &kickstarter);
    }
}
