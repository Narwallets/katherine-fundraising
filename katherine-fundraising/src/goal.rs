use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};

use std::convert::TryFrom;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Goal {
    pub id: u64,
    /// Name of the kickstarter project
    pub name: String,
    /// How many stnear tokens are needed to get this Goal
    pub goal: u128,
    /// End date of the goal
    pub goal_timestamp: Timestamp,
    /// Funders that choose this Goal
    pub funders: Vec<Funder>,
    /// How many tokens are for this 
    pub tokens_to_release: u128,
    /// Date for starting the delivery of stNEARs if the goal was matched
    pub start_delivery_timestamp: Timestamp,
    /// Date for finish the delivery of stNEARs
    pub finish_delivery_timestamp: Timestamp,
}


/// TODO:
impl Goal {
}


#[near_bindgen]
impl KatherineFundraising {
    fn create_goal(&mut self,
        kickstarter_id: u64,
        name: String,
        goal: u128,
        goal_timestamp: Timestamp,
        funders: Vec<Funder>,
        tokens_to_release: u128,
        start_delivery_timestamp: Timestamp,
        finish_delivery_timestamp: Timestamp) {

        let mut k = self.kickstarters.get(kickstarter_id).unwrap_or_default();
        let g = Goal {
            id: u64::try_from(k.goals.len()).unwrap_or_default(),
            name: name,
            goal: goal,
            //TODO: set this to an argument
            goal_timestamp: env::block_timestamp(),
            funders: Vec::new(),
            tokens_to_release: tokens_to_release,
            start_delivery_timestamp: env::block_timestamp(),
            finish_delivery_timestamp: env::block_timestamp(),
        };
        k.goals.push(g);
        self.kickstarters.replace(kickstarter_id, &k);
    }
}
