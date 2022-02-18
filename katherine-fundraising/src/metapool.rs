use crate::*;
use near_sdk::{log, AccountId, Balance, PromiseOrValue};
use near_sdk::Promise;
use near_sdk::serde_json::{json};
use near_sdk::json_types::{ValidAccountId, U128};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

pub use crate::types::*;

impl KatherineFundraising {
    pub(crate) fn take_supporter_stnear(&mut self, supporter: AccountId, amount: Balance) -> Promise {
        Promise::new(self.metapool_contract_address.clone())
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for KatherineFundraising {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Verifying that we were called by fungible token contract that we expect.
        assert_eq!(
            &env::predecessor_account_id(),
            &self.metapool_contract_address,
            "Only supports the one fungible token contract"
        );
        log!("in {} tokens from @{} ft_on_transfer, msg = {}", amount.0, sender_id.as_ref(), msg);

        match self.internal_supporter_deposit(sender_id.as_ref(), &amount.0, msg) {
            Ok(unused_amount) => PromiseOrValue::Value(U128::from(unused_amount)),
            Err(_) => PromiseOrValue::Value(U128::from(amount)) 
        }
    }
}