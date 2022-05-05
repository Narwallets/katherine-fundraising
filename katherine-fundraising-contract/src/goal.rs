use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

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
    pub tokens_to_release_per_stnear: Balance,
    /// Date for starting the delivery of the Kickstarter Tokens if the goal was matched
    pub cliff_timestamp: EpochMillis,
    /// Date to finish the delivery of the Kickstarter Tokens
    pub end_timestamp: EpochMillis,
}

impl Goal {
    pub fn to_json(&self) -> GoalJSON {
        GoalJSON {
            id: self.id.into(),
            name: String::from(&self.name),
            desired_amount: BalanceJSON::from(self.desired_amount),
            unfreeze_timestamp: self.unfreeze_timestamp,
            tokens_to_release_per_stnear: BalanceJSON::from(self.tokens_to_release_per_stnear),
            cliff_timestamp: self.cliff_timestamp,
            end_timestamp: self.end_timestamp,
        }
    }
}

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn internal_create_goal(
        &mut self,
        kickstarter: &mut Kickstarter,
        name: String,
        desired_amount: BalanceJSON,
        unfreeze_timestamp: EpochMillis,
        tokens_to_release_per_stnear: BalanceJSON,
        cliff_timestamp: EpochMillis,
        end_timestamp: EpochMillis,
    ) -> GoalId {
        kickstarter.assert_goal_status();
        kickstarter.assert_before_funding_period();
        kickstarter.assert_number_of_goals(self.max_goals_per_kickstarter);

        let desired_amount = Balance::from(desired_amount);
        let tokens_to_release_per_stnear = Balance::from(tokens_to_release_per_stnear);
        let id = kickstarter.get_number_of_goals();
        assert!(
            kickstarter.deposits_hard_cap >= desired_amount,
            "Desired amount must not exceed the deposits hard cap!"
        );
        assert!(
            kickstarter.max_tokens_to_release_per_stnear >= tokens_to_release_per_stnear,
            "Tokens to release must not exceed the max tokens to release per stNEAR!"
        );
        if id > 0 {
            let last_goal = kickstarter.goals.get((id - 1) as u64).unwrap();
            assert!(
                desired_amount >= last_goal.desired_amount,
                "Next goal cannot have a lower desired amount that the last goal!"
            );
            assert!(
                unfreeze_timestamp <= last_goal.unfreeze_timestamp,
                "Next goal cannot freeze supporter funds any longer than the last goal!"
            );
            assert!(
                tokens_to_release_per_stnear >= last_goal.tokens_to_release_per_stnear,
                "Next goal cannot release less pTOKEN than the last goal!"
            );
        }
        let goal = Goal {
            id,
            name,
            desired_amount,
            unfreeze_timestamp,
            tokens_to_release_per_stnear,
            cliff_timestamp,
            end_timestamp,
        };
        kickstarter.goals.push(&goal);
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
        goal.id
    }

    pub(crate) fn internal_delete_last_goal(&mut self, kickstarter: &mut Kickstarter) {
        kickstarter.assert_goal_status();
        kickstarter.assert_before_funding_period();
        kickstarter.goals.pop();
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
    }
}
