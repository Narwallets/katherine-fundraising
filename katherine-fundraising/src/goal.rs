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
    pub unfreeze_timestamp: EpochMillis,
    /// How many tokens are for this 
    pub tokens_to_release: Balance,
    /// Date for starting the delivery of the Kickstarter Tokens if the goal was matched
    pub cliff_timestamp: EpochMillis,
    /// Date for finish the delivery of the Kickstarter Tokens
    pub end_timestamp: EpochMillis,
}

impl Goal {
    pub fn to_json(&self) -> GoalJSON {
        GoalJSON {
            id: self.id.into(),
            name: String::from(&self.name),
            desired_amount: BalanceJSON::from(self.desired_amount),
            unfreeze_timestamp: self.unfreeze_timestamp,
            tokens_to_release: BalanceJSON::from(self.tokens_to_release),
            cliff_timestamp: self.cliff_timestamp,
            end_timestamp: self.end_timestamp,
        }
    }
}

#[near_bindgen]
impl KatherineFundraising {
    pub fn create_goal(
        &mut self,
        kickstarter_id: KickstarterId,
        name: String,
        desired_amount: BalanceJSON,
        unfreeze_timestamp: EpochMillis,
        tokens_to_release: BalanceJSON,
        cliff_timestamp: EpochMillis,
        end_timestamp: EpochMillis,
    ) -> GoalId {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.assert_only_owner();
        kickstarter.assert_goal_status();
        kickstarter.assert_before_funding_period();
        kickstarter.assert_number_of_goals(self.max_goals_per_kickstarter);
        let goal = Goal {
            id: kickstarter.get_number_of_goals(),
            name,
            desired_amount: Balance::from(desired_amount),
            unfreeze_timestamp,
            tokens_to_release: Balance::from(tokens_to_release),
            cliff_timestamp,
            end_timestamp,
        };
        kickstarter.goals.push(&goal);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
        goal.id
    }

    pub fn delete_last_goal(
        &mut self,
        kickstarter_id: KickstarterId,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.assert_only_owner();
        kickstarter.assert_goal_status();
        kickstarter.assert_before_funding_period();
        kickstarter.goals.pop();
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
    }
}
