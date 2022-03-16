use crate::*;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{U128};

use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Goal {
    pub id: GoalId,
    /// Name of the kickstarter project
    pub name: String,
    /// How many stnear tokens are needed to get this Goal
    pub desired_amount: Balance,
    /// How many tokens are for this 
    pub tokens_to_release: Balance,
    /// Date for starting the delivery of the Kickstarter Tokens if the goal was matched
    pub cliff_timestamp: Milliseconds,
    /// Date for finish the delivery of the Kickstarter Tokens
    pub end_timestamp: Milliseconds,
}

#[near_bindgen]
impl KatherineFundraising {
    pub fn create_goal(
        &mut self,
        kickstarter_id: KickstarterId,
        name: String,
        desired_amount: BalanceJSON,
        tokens_to_release: BalanceJSON,
        cliff_timestamp: Milliseconds,
        end_timestamp: Milliseconds,
    ) -> GoalId {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.assert_only_owner();
        kickstarter.assert_goal_status();
        let goal = Goal {
            id: kickstarter.goals.len() as u8,
            name,
            desired_amount: Balance::from(desired_amount),
            tokens_to_release: Balance::from(tokens_to_release),
            cliff_timestamp,
            end_timestamp,
        };
        kickstarter.goals.push(&goal);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
        goal.id
    }
}
