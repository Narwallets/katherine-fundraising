use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault};
use near_sdk::collections::{UnorderedMap, Vector};

use crate::iou_note::IOUNoteDenomination;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Kickstarter {
    /// Unique ID identifier
    pub id: KickstarterId,

    /// Name of the kickstarter project
    pub name: String,

    /// TODO: documentation for slug
    pub slug: String,

    /// TODO: Goals
    pub goals: Vector<Goal>,

    pub winner_goal_id: Option<u8>,

    /// Katherine fee is denominated in Kickstarter Tokens.
    pub katherine_fee: Option<Balance>,

    /// TODO: Supporters, IS THIS NECESARY IF SUPPORTERS ARE ALREADY IN DEPOSITS?
    pub supporters: Vec<Supporter>,

    /// Deposits during the funding period.
    pub deposits: UnorderedMap<SupporterId, Balance>,

    /// TODO: Owner
    pub owner_id: AccountId,

    /// True if the kickstart project is active and waiting for funding.
    pub active: bool,

    /// True if the kickstart project met the goals
    pub successful: bool,

    /// Spot near
    pub stnear_value_in_near: Option<Balance>,

    /// Creation date of the project
    pub creation_timestamp: Timestamp,

    /// Finish Timestamp of the project. This date is set if the project met its goals
    pub finish_timestamp: Timestamp,

    /// Opening date to recieve deposits from supporters. TODO: more detail here
    pub open_timestamp: Timestamp,

    /// Closing date for recieving deposits from supporters. TODO: more detail here
    pub close_timestamp: Timestamp,

    /// How much time the project will be active, this also means how much time the stnear tokens
    /// will be locked
    pub vesting_timestamp: Timestamp,

    /// How much time should pass before releasing the project tokens
    pub cliff_timestamp: Timestamp,

    /// Kickstarter Token contract address.
    pub token_contract_address: AccountId,

    /// Total available and locked deposited tokens by the Kickstarter.
    pub available_tokens: Balance,
    pub locked_tokens: Balance,
}


/// TODO:
impl Kickstarter {
    pub fn get_supporter_ids(&self) -> Vec<AccountId> {
        let mut supporter_ids: Vec<AccountId> = self.deposits.to_vec().into_iter().map(|p| p.0).collect();
        // supporter_ids.sort_unstable();
        // supporter_ids.dedup();
        supporter_ids.to_vec()
    }

    pub fn get_total_supporters(&self) -> u64 {
        self.deposits.len()
    }

    pub fn get_deposits(&self) -> &UnorderedMap<AccountId, Balance> {
        &self.deposits
        // let mut funding_map: UnorderedMap<AccountId, Balance> = UnorderedMap::new(b"A".to_vec());
        // for tx in self.supporter_tickets.clone().into_iter() {
        //     let supporter_id: AccountId = tx.supporter_id;
        //     let ticket_blance: Balance = tx.stnear_amount;
        //     let current_total: Balance = match funding_map.get(&supporter_id) {
        //         Some(total) => total,
        //         None => 0,
        //     };
        //     let new_total: Balance = current_total + ticket_blance;
        //     funding_map.insert(&supporter_id, &new_total);
        // }
        // funding_map
    }

    pub fn get_total_deposited_amount(&self) -> Balance {
        let total_amount: Vec<Balance> = self.deposits.to_vec().into_iter().map(|p| p.1).collect();
        total_amount.into_iter().sum()
    }

    pub fn evaluate_goals(&mut self) -> bool {
        if let None = self.winner_goal_id {
            let total_deposits = self.get_total_deposited_amount();
            let mut achieved_goals: Vec<Goal> = self.goals
                .to_vec()
                .into_iter()
                .filter(|goal| goal.desired_amount <= total_deposits)
                .collect();

            if achieved_goals.len() > 0 {
                achieved_goals.sort_by_key(|goal| goal.desired_amount);
                let winner_goal = achieved_goals.last().unwrap();
                self.winner_goal_id = Some(winner_goal.id as u8);
                return true;
            } else {
                return false;
            }

        } else {
            panic!("Kickstarter already has a winning goal!");
        }
    }

    pub fn simple_evaluate_goals(&self) -> bool {
        let total_deposits = self.get_total_deposited_amount();
        self.goals
            .iter()
            .any(|goal| goal.desired_amount >= total_deposits)
    }

    pub fn get_goal(&self) -> Goal {
        self.goals
            .get(self.winner_goal_id.expect("No goal defined") as u64)
            .expect("Incorrect goal index") 
    }

    // WARNING: This is only callable by Katherine.
    pub fn update_supporter_deposits(&mut self, supporter_id: &AccountId, amount: &Balance) {
        let current_supporter_deposit = match self.deposits.get(&supporter_id) {
            Some(total) => total,
            None => 0,
        };
        let new_total: Balance = current_supporter_deposit + amount;
        self.deposits.insert(&supporter_id, &new_total);
    }

    pub fn convert_stnear_to_near(&self, amount_in_stnear: &Balance) -> Balance {
        // WARNING: This operation must be enhaced.
        let rate = self.stnear_value_in_near.expect("Conversion rate has not been stablished!");
        let amount_in_near = amount_in_stnear / rate;
        amount_in_near
    }

    pub fn get_tokens_to_release(&self) -> Balance {
        self.get_goal().tokens_to_release
    }

    pub fn get_total_rewards_for_supporters(&self) -> Balance {
        self.get_tokens_to_release() - self.katherine_fee.expect("Katherine fee must be denominated at goal evaluation")
    }

    pub fn set_katherine_fee(&self) -> Balance {
        unimplemented!()
    }

    pub fn convert_stnear_to_token_shares(&self, amount_in_stnear: &Balance) -> Balance {
        // WARNING: This operation must be enhaced.
        // This is a Rule of Three calculation to get the shares.
        let tokens_rewards = self.get_total_rewards_for_supporters();
        let total_support = self.get_total_deposited_amount();
        amount_in_stnear * tokens_rewards / total_support
    }

    pub fn get_token_denomination(&self) -> IOUNoteDenomination {
        self.get_goal().tokens_denomination 
    }

    pub fn get_reward_cliff_timestamp(&self) -> Timestamp {
        self.get_goal().cliff_timestamp
    }

    pub fn get_reward_end_timestamp(&self) -> Timestamp {
        self.get_goal().end_timestamp 
    }
}
