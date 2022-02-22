use crate::*;
use near_sdk::{AccountId, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault};
use near_sdk::collections::{UnorderedMap};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Kickstarter {
    /// Unique ID identifier
    pub id: KickstarterId,

    /// Name of the kickstarter project
    pub name: String,

    /// TODO: documentation for slug
    pub slug: String,

    /// TODO: Goals
    pub goals: Vec<Goal>,

    /// TODO: All the Supporter tickets of the project
    pub supporter_tickets: Vec<Ticket>,

    /// TODO: Owner
    pub owner: AccountId,

    /// True if the kickstart project is active and waiting for funding.
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
/*impl Default for Kickstarter {
    fn default() -> Self {
        Self {
        }
    }
}*/

/// TODO:
impl Kickstarter {
    pub fn get_supporter_ids(&self) -> Vec<AccountId> {
        let mut supporter_ids: Vec<AccountId> = self.supporter_tickets.clone().into_iter().map(|p| p.supporter_id).collect();
        supporter_ids.sort_unstable();
        supporter_ids.dedup();
        supporter_ids
    }

    pub fn get_supporters_funding_map(&self) -> UnorderedMap<AccountId, Balance> {
        let mut funding_map: UnorderedMap<AccountId, Balance> = UnorderedMap::new(b"A".to_vec());
        for tx in self.supporter_tickets.clone().into_iter() {
            let supporter_id: AccountId = tx.supporter_id;
            let ticket_blance: Balance = tx.stnear_amount;
            let current_total: Balance = match funding_map.get(&supporter_id) {
                Some(total) => total,
                None => 0,
            };
            let new_total: Balance = current_total + ticket_blance;
            funding_map.insert(&supporter_id, &new_total);
        }
        funding_map
    }

    pub fn get_total_amount(&self) -> Balance {
        let total_amount: Vec<Balance> = self.supporter_tickets.clone().into_iter().map(|p| p.stnear_amount).collect();
        total_amount.into_iter().sum()
    }
}
