use crate::*;
use near_sdk::{AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Kickstarter {
    // Unique ID identifier
    pub id: KickstarterId,
    // Name of the kickstarter project
    pub name: String,
    // TODO: documentation for slug
    pub slug: String,
    // TODO: Goals
    pub goals: Vector<Goal>,
    pub winner_goal_id: Option<u8>,
    // Katherine fee is denominated in Kickstarter Tokens.
    pub katherine_fee: Option<Balance>,
    // TODO: Supporters, IS THIS NECESARY IF SUPPORTERS ARE ALREADY IN DEPOSITS?
    // pub supporters: Vec<Supporter>,
    pub total_supporters: u32,
    // Deposits during the funding period.
    pub deposits: UnorderedMap<SupporterId, Balance>,
    pub withdraw: UnorderedMap<SupporterId, Balance>,
    pub total_deposited: Balance,
    // TODO: Owner
    pub owner_id: AccountId,
    // True if the kickstart project is active and waiting for funding.
    pub active: bool,
    // True if the kickstart project met the goals
    pub successful: Option<bool>,
    // Spot stnear price at freeze and unfreeze.
    pub stnear_price_at_freeze: Option<Balance>,
    pub stnear_price_at_unfreeze: Option<Balance>,
    // Creation date of the project
    pub creation_timestamp: EpochMillis,
    // Opening date to recieve deposits from supporters. TODO: more detail here
    pub open_timestamp: EpochMillis,
    // Closing date for recieving deposits from supporters. TODO: more detail here
    pub close_timestamp: EpochMillis,
    // Kickstarter Token contract address.
    pub token_contract_address: AccountId,
    // Total available and locked deposited tokens by the Kickstarter.
    pub available_reward_tokens: Balance,
    pub locked_reward_tokens: Balance,
}

impl Kickstarter {
    #[inline]
    pub fn assert_goal_status(&self) {
        assert!(self.winner_goal_id.is_none(), "Kickster already has a winning goal.");
    }

    #[inline]
    pub(crate) fn assert_only_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "only allowed for admin");
    }

    #[inline]
    pub(crate) fn assert_before_funding_period(&self) {
        assert!(get_current_epoch_millis() < self.open_timestamp, "Action not allow after funding period is open!");
    }

    #[inline]
    pub(crate) fn assert_number_of_goals(&self, max_number: u8) {
        assert!(max_number >= self.get_number_of_goals(), "Too many goals!");
    }

    #[inline]
    pub(crate) fn assert_unfreezed_funds(&self) {
        assert!(self.get_goal().unfreeze_timestamp < get_current_epoch_millis(), "Assets are still freezed.");
    }
}

impl Kickstarter {
    pub fn get_supporter_ids(&self) -> Vec<AccountId> {
        let supporter_ids: Vec<AccountId> = self.deposits.to_vec().into_iter().map(|p| p.0).collect();
        // supporter_ids.sort_unstable();
        // supporter_ids.dedup();
        supporter_ids.to_vec()
    }

    pub fn get_total_supporters(&self) -> u32 {
        self.deposits.len() as u32
    }

    pub fn get_deposit(&self, supporter_id: &SupporterId) -> Balance {
        self.deposits.get(&supporter_id).expect("Supporter is not part of Kickstarter!")
    }

    pub fn get_withdraw(&self, supporter_id: &SupporterId) -> Balance {
        match self.withdraw.get(&supporter_id) {
            Some(amount) => amount,
            None => 0,
        }
    }

    /// Deprecated!
    pub fn get_total_deposited_amount(&self) -> Balance {
        self.total_deposited
        //let total_amount: Vec<Balance> = self.deposits.to_vec().into_iter().map(|p| p.1).collect();
        //total_amount.into_iter().sum()
    }

    pub fn get_achieved_goal(&mut self) -> Option<Goal> {
        let mut achieved_goals: Vec<Goal> = self.goals
            .iter()
            .filter(|goal| goal.desired_amount <= self.total_deposited)
            .collect();
        if achieved_goals.len() > 0 {
            achieved_goals.sort_by_key(|goal| goal.desired_amount);
            let winner_goal_id = achieved_goals.last().unwrap().id;
            let winner_goal = self.goals.get(winner_goal_id as u64).unwrap();
            return Some(winner_goal);
        } else {
            return None;
        }
    }

    pub fn any_achieved_goal(&self) -> bool {
        self.goals
            .iter()
            .any(|goal| goal.desired_amount <= self.total_deposited)
    }

    pub fn get_goal(&self) -> Goal {
        self.goals
            .get(self.winner_goal_id.expect("No goal defined") as u64)
            .expect("Incorrect goal index") 
    }

    pub fn get_goal_by_id(&self, goal_id: GoalId) -> Goal {
        self.goals.get(goal_id as u64).expect("Goal not found!")
    }

    // WARNING: This is only callable by Katherine.
    pub(crate) fn update_supporter_deposits(&mut self, supporter_id: &AccountId, amount: &Balance) {
        let current_supporter_deposit = match self.deposits.get(&supporter_id) {
            Some(total) => total,
            None => {
                self.total_supporters += 1;
                0
            },
        };
        let new_total: Balance = current_supporter_deposit + amount;
        self.deposits.insert(&supporter_id, &new_total);
    }

    pub fn convert_stnear_to_near(&self, amount_in_stnear: &Balance) -> Balance {
        // WARNING: This operation must be enhaced.
        let rate = self.stnear_price_at_freeze.expect("Conversion rate has not been stablished!");
        let amount_in_near = amount_in_stnear / rate;
        amount_in_near
    }

    pub fn get_tokens_to_release(&self) -> Balance {
        self.get_goal().tokens_to_release
    }

    pub fn get_total_rewards_for_supporters(&self) -> Balance {
        self.get_tokens_to_release() - self.katherine_fee.expect("Katherine fee must be denominated at goal evaluation")
    }

    pub fn set_katherine_fee(&mut self, katherine_fee_percent: BasisPoints, goal: &Goal) {
        let katherine_fee: Balance = proportional(
            katherine_fee_percent as u128,
            goal.tokens_to_release,
            BASIS_POINTS
        );
        self.katherine_fee = Some(katherine_fee);
    }

    pub fn set_stnear_value_in_near(&mut self) {
        unimplemented!()
    }

    pub fn convert_stnear_to_token_shares(&self, amount_in_stnear: &Balance) -> Balance {
        // WARNING: This operation must be enhaced.
        // This is a Rule of Three calculation to get the shares.
        let tokens_rewards = self.get_total_rewards_for_supporters();
        amount_in_stnear * tokens_rewards / self.total_deposited
    }

    pub fn get_reward_cliff_timestamp(&self) -> EpochMillis {
        self.get_goal().cliff_timestamp
    }

    pub fn get_reward_end_timestamp(&self) -> EpochMillis {
        self.get_goal().end_timestamp 
    }

    pub fn get_number_of_goals(&self) -> u8 {
        self.goals.len() as u8
    }

    pub fn to_json(&self) -> KickstarterJSON {
        KickstarterJSON {
            id: self.id.into(),
            total_supporters: self.total_supporters,
            open_timestamp: self.open_timestamp,
            close_timestamp: self.close_timestamp,
        }
    }
}
