use core::panic;

use crate::*;
use near_sdk::{log, AccountId, Balance, PromiseOrValue, env, ext_contract, near_bindgen, PromiseResult};
use near_sdk::Promise;
use near_sdk::serde_json::{json};
use near_sdk::json_types::{ValidAccountId, U128};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

pub use crate::types::*;

// define the methods we'll use on the other contract
#[ext_contract(ext_metapool)]
pub trait MetaPool {
    fn get_contract_state(&self) -> GetContractStateResult;
}

// define methods we'll use as callbacks on our contract
#[ext_contract(ext_self)]
pub trait KatherineFundraising {
    fn process_metapool_state_result(&self);
}

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

#[near_bindgen]
impl KatherineFundraising {
    pub fn set_stnear_value_in_near(&self, kickstarter: &mut Kickstarter) {
        // Invoke a method on another contract
        // This will send an ActionReceipt to the shard where the contract lives.
        ext_metapool::get_contract_state(
            &self.metapool_contract_address,
            0, // yocto NEAR to attach
            5_000_000_000_000 // gas to attach
        )
        // After the smart contract method finishes a DataReceipt will be sent back
        // .then registers a method to handle that incoming DataReceipt
        .then(ext_self::process_metapool_state_result(
            &env::current_account_id(), // this contract's account id
            0, // yocto NEAR to attach to the callback
            5_000_000_000_000 // gas to attach to the callback
        ));
    }

    pub fn process_metapool_state_result(&self, kickstarter: &mut Kickstarter) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 100000 {
                    "Wow!".to_string()
                } else {
                    "Hmmmm".to_string()
                }
            },
        };
    }
}