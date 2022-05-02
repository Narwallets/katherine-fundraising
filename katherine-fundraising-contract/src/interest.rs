use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId};

use crate::interface::*;

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn kickstarter_withdraw(
        &mut self,
        kickstarter: &mut Kickstarter,
        price_at_unfreeze: Balance
    ) {
        let price_at_freeze = kickstarter.stnear_price_at_freeze.expect("stnear price at freeze not defined");
        assert!(price_at_unfreeze > price_at_freeze, "stNear price has not been updated, please wait!");
        available_interest = kickstarter.calculate_interest(price_at_freeze, price_at_unfreeze);






        let price_increment = price_at_unfreeze - price_at_freeze;

        let max_withdraw = (U256::from(price_increment) * U256::from(kickstarter.total_deposited)).as_u128() - kickstarter.kickstarter_withdraw;        
        assert!(max_withdraw >= amount, "amount to withdraw exceeds balance");

        if is_close(amount, max_withdraw) {
            amount = max_withdraw;
        }
        kickstarter.kickstarter_withdraw += amount;
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
        nep141_token::ft_transfer_call(
            convert_to_valid_account_id(env::predecessor_account_id()),
            amount.into(),
            None,
            "kickstarter stnear withdraw".to_owned(),
            &self.metapool_contract_address,
            0,
            GAS_FOR_FT_TRANSFER
        )
        .then(ext_self_kickstarter::kickstarter_withdraw_resolve_transfer(
            kickstarter.id.into(), 
            amount.into(),
            &env::current_account_id(),
            0,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_FT_TRANSFER
        ));
    }



    pub(crate) fn kickstarter_withdraw_before_unfreeze(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
    ) {
        assert!(
            !kickstarter.funds_can_be_unfreezed(),
            "Unfreeze funds before interest withdraw!"
        );
        // Get stNear price from metapool.
        ext_self_metapool::get_st_near_price(
            &self.metapool_contract_address,
            0,
            GAS_FOR_GET_STNEAR,
        )
        .then(ext_self_kickstarter::kickstarter_withdraw_callback(
            kickstarter_id,
            amount.into(),
            &env::current_account_id(),
            0,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_GET_STNEAR,
        ));
    }

    #[private]
    pub fn kickstarter_withdraw_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
        #[callback] st_near_price: U128,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        self.kickstarter_withdraw(&mut kickstarter, st_near_price.into(), amount.into());
    }

    #[private]
    pub fn kickstarter_withdraw_resolve_transfer(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount.0
                );
                self.internal_restore_kickstarter_withdraw(amount.into(), kickstarter_id.into())
            }
        }
    }

    

    pub(crate) fn internal_restore_kickstarter_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId
    ){
        let mut kickstarter = self
        .kickstarters
        .get(kickstarter_id.into())
        .expect("kickstarter not found");
        assert!(kickstarter.kickstarter_withdraw <= amount, "withdrawn amount is higher than expected");
        kickstarter.kickstarter_withdraw -= amount;
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }
}

impl Kickstarter {
    fn calculate_interest(&self, price_at_freeze: Balance, price_at_freeze: Balance) -> Balance {

    }
}