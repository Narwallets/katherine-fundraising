use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, Timestamp, PromiseResult
};

mod constants;
mod errors;
pub mod goal;
mod internal;
pub mod kickstarter;
mod metapool;
pub mod supporter;
mod types;
pub mod utils;

use crate::{constants::*, goal::*, kickstarter::*, metapool::*, supporter::*, types::*, utils::*};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {
    pub owner_id: AccountId,
    pub supporters: UnorderedMap<AccountId, Supporter>,
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
    pub fn new(
        owner_id: AccountId,
        staking_goal: Balance,
        min_deposit_amount: Balance,
        metapool_contract_address: AccountId,
        fee_percent: f32,
    ) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(b"A".to_vec()),
            kickstarters: Vector::new(b"Kickstarters".to_vec()),
            total_available: 0,
            min_deposit_amount,
            metapool_contract_address,
            katherine_fee_percent: fee_percent,
        }
    }

    #[payable]
    pub fn deposit_and_stake(&mut self, amount: Balance) {
        unimplemented!();
        let supporter = env::predecessor_account_id();
        // let supporter_stnear: Promise = self.take_supporter_stnear(supporter, amount);
        self.internal_deposit(amount);
    }

    pub fn withdraw_kickstarter_tokens(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON){
        //WIP
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let account = env::predecessor_account_id();
        self.internal_withdraw(amount.into(), kickstarter_id.into(), &account);
        metapool_token::ft_transfer_call(
            account.clone(),
            amount,
            Some("withdraw from kickstarter".to_string()),
            &self.metapool_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        // restore user balance on error
        .then(ext_self_metapool::return_tokens_callback(
            account.clone(),
            kickstarter_id,
            amount,
            &env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER
        ));
    }

    #[private]
    pub fn return_tokens_callback(&mut self, user: AccountId, kickstarter_id: KickstarterIdJSON, amount: U128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount.0
                );
                self.restore_withdraw(amount.into(), kickstarter_id.into(), user)
            }
        }
    }

    /*****************************/
    /* staking-pool View methods */
    /*****************************/

    pub fn get_supporter_available_balance(&self, supporter_id: AccountId) -> U128 {
        let supporter = self.internal_get_supporter(&supporter_id);
        supporter.available.into()
    }

    pub fn get_contract_total_available(&self) -> U128 {
        self.total_available.into()
    }

    /************************/
    /*    Robot methods     */
    /************************/

    /// returns both successfull and unsuccessfull kickstarter ids from a closed subgroup
    pub fn get_kickstarters_to_process(
        &self,
        from_index: U64,
        limit: U64,
    ) -> Option<(KickstarterStatusJSON, KickstarterStatusJSON)> {
        let kickstarters_len = self.kickstarters.len();
        let start: u64 = from_index.into();

        if start >= kickstarters_len {
            return None;
        }
        let mut successfull: Vec<KickstarterIdJSON> = Vec::new();
        let mut unsuccessfull: Vec<KickstarterIdJSON> = Vec::new();

        for index in start..std::cmp::min(start + u64::from(limit), kickstarters_len) {
            let kickstarter = self
                .kickstarters
                .get(index)
                .expect("internal error, kickstarter ID is out of range");
            if kickstarter.active && kickstarter.close_timestamp <= get_epoch_millis() {
                let any_goal = kickstarter.any_achieved_goal();
                if any_goal {
                    successfull.push(KickstarterIdJSON::from(kickstarter.id))
                } else {
                    unsuccessfull.push(KickstarterIdJSON::from(kickstarter.id))
                }
            }
        }
        return Some((
            KickstarterStatusJSON {
                status: KickstarterResult::Successful,
                ids: successfull,
            },
            KickstarterStatusJSON {
                status: KickstarterResult::Unsuccessfull,
                ids: unsuccessfull,
            },
        ));
    }

    pub fn process_kickstarter(&mut self, kickstarter_id: KickstarterIdJSON) {
        let mut kickstarter = self.internal_get_kickstarter(KickstarterId::from(kickstarter_id));
        let winning_goal = kickstarter.get_achieved_goal();

        if kickstarter.successful != None {
            if let Some(goal) = winning_goal {
                assert!(
                    kickstarter.available_reward_tokens >= goal.tokens_to_release,
                    "not enough available reward tokens to back the supporters rewards"
                );
                kickstarter.winner_goal_id = Some(goal.id);
                kickstarter.active = false;
                kickstarter.successful = Some(true);
                kickstarter.set_katherine_fee();
                kickstarter.set_stnear_value_in_near();
                log!("kickstarter successfully activated");
            } else {
                kickstarter.active = false;
                kickstarter.successful = Some(false);
                log!("kickstarter successfully deactivated");
            }
        } else {
            panic!("kickstarter already activated");
        }
    }

    /*****************************/
    /*   Kickstarter functions   */
    /*****************************/

    /// Creates a new kickstarter entry in persistent storage
    pub fn create_kickstarter(
        &mut self,
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
        //TODO only allow admin access
        let kickstarter = Kickstarter {
            id: self.kickstarters.len(),
            name,
            slug,
            goals: Vector::new(b"Goal".to_vec()),
            winner_goal_id: None,
            katherine_fee: None,
            supporters: Vec::new(),
            total_supporters: 0,
            deposits: UnorderedMap::new(b"A".to_vec()),
            total_deposited: 0,
            owner_id,
            active: true,
            successful: None,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
        };

        self.kickstarters.push(&kickstarter);
    }

    #[allow(unused)]
    pub fn delete_kickstarter(&mut self, id: KickstarterId) {
        panic!("Kickstarter must not be deleted!");
    }

    /// Update a kickstarter
    pub fn update_kickstarter(
        &mut self,
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
        let old_kickstarter = self
            .kickstarters
            .get(id)
            .expect("Kickstarter Id not found!");
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
            total_supporters: 0,
            deposits: UnorderedMap::new(b"A".to_vec()),
            total_deposited: 0,
            owner_id,
            active: true,
            successful: None,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
        };

        self.kickstarters.replace(id, &kickstarter);
    }

    /********************/
    /*   View methods   */
    /********************/

    pub fn get_total_kickstarters(&self) -> U64 {
        return self.kickstarters.len().into();
    }
}
