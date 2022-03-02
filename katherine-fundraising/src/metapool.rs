use core::panic;

use crate::*;
use near_sdk::{log, AccountId, Balance, PromiseOrValue};
use near_sdk::Promise;
use near_sdk::serde_json::{json};
use near_sdk::json_types::{ValidAccountId, U128};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

pub use crate::types::*;

#[near_bindgen]
impl FungibleTokenReceiver for KatherineFundraising {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let kickstarter_id = match msg.parse::<KickstarterId>() {
            Ok(_id) => _id,
            Err(_) => panic!("Invalid Kickstarter id.".into()),
        };

        let mut kickstarter: Kickstarter = match self.kickstarters.get(kickstarter_id) {
            Some(kickstarter) => kickstarter,
            None => panic!("Kickstarter id not found.".into()),
        };

        let result = if env::predecessor_account_id() == self.metapool_contract_address {
            // Deposit is in stNEAR.
            log!("DEPOSIT: {} stNEAR deposited from {} to Kickstarter id {}", amount.0, sender_id.as_ref(), msg);
            self.internal_supporter_deposit(sender_id.as_ref(), &amount.0, &mut kickstarter) 
        } else {
            // Deposit is in a Kickstarter Token.
            log!("DEPOSIT: {} tokens deposited from {} to Kickstarter id {}", amount.0, sender_id.as_ref(), msg);
            self.internal_kickstarter_deposit(&amount.0, &mut kickstarter)
        };

        match result {
            Ok(unused_amount) => PromiseOrValue::Value(U128::from(unused_amount)),
            Err(_) => PromiseOrValue::Value(U128::from(amount.0)) 
        }
    }
}