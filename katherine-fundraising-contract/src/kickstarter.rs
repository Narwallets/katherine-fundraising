use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Kickstarter {
    // Unique ID identifier
    pub id: KickstarterId,
    // Name of the kickstarter project
    pub name: String,
    // TODO: documentation for slug
    pub slug: String,
    pub goals: Vector<Goal>,
    pub owner_id: AccountId,
    pub winner_goal_id: Option<u8>,
    // Katherine fee is denominated in Kickstarter Tokens.
    pub katherine_fee: Option<Balance>,
    // This is the Kickstarter Tokens that will be used to pay the Supporters.
    // To make a Kickstarter successful:
    // katherine_fee + total_tokens_to_release > available_reward_tokens
    pub total_tokens_to_release: Option<Balance>,
    // Deposits during the funding period.
    pub deposits: UnorderedMap<SupporterId, Balance>,
    pub withdraw: UnorderedMap<SupporterId, Balance>,

    // Important Note: the kickstarter.total_deposited variable will only increase or decrease within
    // the funding period. After the project evaluation, this value will stay CONSTANT to store a 
    // record of the achieved funds, even after all stNear will be withdraw from the kickstarter.
    pub total_deposited: Balance,
    // Total deposited hard cap. Supporters cannot deposit more than.
    pub deposits_hard_cap: Balance,
    pub max_tokens_to_release_per_stnear: Balance,
    pub enough_reward_tokens: bool,
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
    pub kickstarter_withdraw: Balance
}

impl Kickstarter {
    #[inline]
    pub fn assert_goal_status(&self) {
        assert!(
            self.winner_goal_id.is_none(),
            "Kickster already has a winning goal."
        );
    }

    #[inline]
    pub(crate) fn assert_kickstarter_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "only allowed for admin"
        );
    }

    #[inline]
    pub(crate) fn assert_before_funding_period(&self) {
        assert!(
            get_current_epoch_millis() < self.open_timestamp,
            "Action not allow after funding period is open!"
        );
    }

    #[inline]
    pub(crate) fn assert_number_of_goals(&self, max_number: u8) {
        assert!(max_number >= self.get_number_of_goals(), "Too many goals!");
    }

    #[inline]
    pub(crate) fn assert_funds_can_be_unfreezed(&self) {
        assert!(
            self.funds_can_be_unfreezed(),
            "Assets are still freezed."
        );
    }

    #[inline]
    pub(crate) fn assert_funds_must_be_unfreezed(&self) {
        self.assert_funds_can_be_unfreezed();
        assert!(
            self.stnear_price_at_unfreeze.is_some(),
            "Price at unfreeze is not defined. Please unfreeze kickstarter funds with fn: unfreeze_kickstarter_funds!"
        );
    }

    #[inline]
    pub(crate) fn assert_timestamps(&self) {
        assert!(
            self.open_timestamp >= get_current_epoch_millis(),
            "Incorrect open timestamp!"
        );
        assert!(
            self.close_timestamp >= self.open_timestamp,
            "Incorrect close timestamp!"
        );
    }

    #[inline]
    pub(crate) fn assert_within_funding_period(&self) {
        assert!(
            self.is_within_funding_period(),
            "Not within the funding period."
        );
    }

    pub(crate) fn assert_enough_reward_tokens(&self) {
        assert!(
            self.enough_reward_tokens,
            "Supporters cannot deposit until the Kickstarter covers the required rewards!"
        );
    }
}

impl Kickstarter {
    pub fn is_within_funding_period(&self) -> bool {
        let now = get_current_epoch_millis();
        now < self.close_timestamp && now >= self.open_timestamp
    }

    pub fn funds_can_be_unfreezed(&self) -> bool {
        self.get_winner_goal().unfreeze_timestamp < get_current_epoch_millis()
    }

    pub fn get_total_supporters(&self) -> u32 {
        self.deposits.len() as u32
    }

    pub fn get_deposit(&self, supporter_id: &SupporterId) -> Balance {
        self.deposits
            .get(&supporter_id)
            .expect("Supporter is not part of Kickstarter!")
    }

    pub fn get_withdraw(&self, supporter_id: &SupporterId) -> Balance {
        match self.withdraw.get(&supporter_id) {
            Some(amount) => amount,
            None => 0,
        }
    }

    // pub fn get_achieved_goal(&mut self) -> Option<Goal> {
    //     let mut iter = self.goals.iter();
    //     let result = iter.next();
    //     if result == None {
    //         return None;
    //     } else {
    //         let mut winner_goal = result.unwrap();
    //         for goal in iter {
    //             if goal.desired_amount > winner_goal.desired_amount && goal.desired_amount >= self.total_deposited {
    //                 winner_goal = goal;
    //             }
    //         }
            
    //         if winner_goal.desired_amount >= self.total_deposited {
    //             return Some(winner_goal)
    //         }
    //         else{
    //             return None
    //         }
    //     }
    // }

    // TODO: The code above is not working, we were trying to make more efficient the code bellow.
    pub fn get_achieved_goal(&mut self) -> Option<Goal> {
        let mut achieved_goals: Vec<Goal> = self
            .goals
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

    pub fn get_winner_goal(&self) -> Goal {
        self.goals
            .get(self.winner_goal_id.expect("No goal defined") as u64)
            .expect("Incorrect goal index")
    }

    pub fn get_goal_by_id(&self, goal_id: GoalId) -> Goal {
        self.goals.get(goal_id as u64).expect("Goal not found!")
    }

    pub(crate) fn update_supporter_deposits(&mut self, supporter_id: &AccountId, amount: &Balance) {
        let current_supporter_deposit = match self.deposits.get(&supporter_id) {
            Some(total) => total,
            None => 0,
        };
        let new_total: Balance = current_supporter_deposit + amount;
        self.deposits.insert(&supporter_id, &new_total);
    }

    pub fn get_tokens_to_release(&self) -> Balance {
        self.get_winner_goal().tokens_to_release
    }

    pub fn get_total_rewards_for_supporters(&self) -> Balance {
        self.get_tokens_to_release()
            - self
                .katherine_fee
                .expect("Katherine fee must be denominated at goal evaluation")
    }

    pub fn get_number_of_goals(&self) -> u8 {
        self.goals.len() as u8
    }

    pub fn to_json(&self) -> KickstarterJSON {
        KickstarterJSON {
            id: self.id.into(),
            total_supporters: self.deposits.len() as u32,
            total_deposited: BalanceJSON::from(self.total_deposited),
            open_timestamp: self.open_timestamp,
            close_timestamp: self.close_timestamp,
        }
    }

    pub fn to_details_json(&self) -> KickstarterDetailsJSON {
        let mut goals: Vec<GoalJSON> = Vec::new();
        for goal in self.goals.iter() {
            goals.push(goal.to_json());
        }
        let stnear_price_at_freeze = if self.stnear_price_at_freeze.is_some() {
            BalanceJSON::from(self.stnear_price_at_freeze.unwrap())
        } else {
            BalanceJSON::from(0)
        };
        let stnear_price_at_unfreeze = if self.stnear_price_at_unfreeze.is_some() {
            BalanceJSON::from(self.stnear_price_at_unfreeze.unwrap())
        } else {
            BalanceJSON::from(0)
        };
        KickstarterDetailsJSON {
            id: self.id.into(),
            total_supporters: self.deposits.len() as u32,
            total_deposited: BalanceJSON::from(self.total_deposited),
            stnear_price_at_freeze,
            stnear_price_at_unfreeze,
            open_timestamp: self.open_timestamp,
            close_timestamp: self.close_timestamp,
            token_contract_address: self.token_contract_address.clone(),
            goals,
            active: self.active,
            successful: self.successful,
            winner_goal_id: self.winner_goal_id,
        }
    }
}
