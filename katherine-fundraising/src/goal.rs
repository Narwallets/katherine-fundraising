use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{U128};

use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Goal {
    pub id: u8,
    /// Name of the kickstarter project
    pub name: String,
    /// How many stnear tokens are needed to get this Goal
    pub desired_amount: u128,
    /// End date of the goal
    pub goal_timestamp: Timestamp,
    /// How many tokens are for this 
    pub tokens_to_release: u128,
    /// Kickstarter Token denomination.
    pub tokens_denomination: String,
    /// Date for starting the delivery of the Kickstarter Tokens if the goal was matched
    pub cliff_timestamp: Timestamp,
    /// Date for finish the delivery of the Kickstarter Tokens
    pub end_timestamp: Timestamp,
}

#[near_bindgen]
impl KatherineFundraising {
    fn create_goal(&mut self,
        kickstarter_id: KickstarterId,
        name: String,
        desired_amount: U128,
        goal_timestamp: Timestamp,
        tokens_to_release: U128,
        tokens_denomination: String,
        start_delivery_timestamp: Timestamp,
        finish_delivery_timestamp: Timestamp) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        only_kickstarter_admin(&kickstarter);
        assert!(kickstarter.successful.is_none(), "cannot create goal after one is reached");
        let g = Goal {
            id: u8::try_from(kickstarter.goals.len()).unwrap_or_default(),
            name: name,
            desired_amount: desired_amount.into(),
            //TODO: set this to an argument
            goal_timestamp: env::block_timestamp(),
            tokens_to_release: tokens_to_release.into(),
            tokens_denomination,
            cliff_timestamp: env::block_timestamp(),
            end_timestamp: env::block_timestamp(),
        };
        kickstarter.goals.push(&g);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
    }
}
