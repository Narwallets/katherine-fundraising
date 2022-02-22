use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Kickstarter {
    /// Unique ID identifier
    pub id: u64,

    /// Name of the kickstarter project
    pub name: String,

    /// TODO: documentation for slug
    pub slug: String,

    /// TODO: Goals
    pub goals: Vec<Goal>,

    /// TODO: Funders
    pub funders: Vec<Funder>,

    /// TODO: All the Supporter tickets of the project
    pub supporter_tickets: Vec<Ticket>,

    /// TODO: Owner
    pub owner: AccountId,

    /// True if the kickstart project is active
    pub active: bool,

    /// True if the kickstart project met the goals
    pub succesful: bool,

    /// Creation date of the project
    pub creation_timestamp: Timestamp,

    /// Finish Timestamp of the project. This date is set if the project met its goals
    pub finish_timestamp: Timestamp,

    /// Opening date to recieve deposits from supporters. TODO: more detail here
    pub open_timestamp: Timestamp,

    /// Closing date for recieving deposits from supporters. TODO: more detail here
    pub close_timestamp: Timestamp,

    /// How much time the project will be active, this also means how much time the stnear tokens
    /// will be blocked
    pub vesting_timestamp: Timestamp,

    /// How much time should pass before releasing the project tokens
    pub cliff_timestamp: Timestamp,
}

/// Kickstarter project
/// TODO...
impl Default for Kickstarter {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            slug: "".to_string(),
            goals: Vec::new(),
            funders: Vec::new(),
            supporter_tickets: Vec::new(),
            owner: env::predecessor_account_id(),
            active: false,
            succesful: false,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp: env::block_timestamp(),
            open_timestamp: env::block_timestamp(),
            close_timestamp: env::block_timestamp(),
            vesting_timestamp: env::block_timestamp(),
            cliff_timestamp: env::block_timestamp(),
        }
    }
}


/// TODO:
impl Kickstarter {
    pub fn get_supporter_ids(&self) -> Vec<AccountId> {
        let mut supporter_ids: Vec<AccountId> = self.supporter_tickets.clone().into_iter().map(|p| p.supporter_id).collect();
        supporter_ids.sort_unstable();
        supporter_ids.dedup();
        supporter_ids
    }

    pub fn get_total_amount(&self) -> Balance {
        let total_amount: Vec<Balance> = self.supporter_tickets.clone().into_iter().map(|p| p.stnear_amount).collect();
        total_amount.into_iter().sum()
    }
}
