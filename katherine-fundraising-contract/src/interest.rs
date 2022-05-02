use crate::*;
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::{near_bindgen, AccountId};

use crate::interface::*;

/// Katherine Math:
/// The following 5 functions describe the math to calculate the **Total stNear for Kickstarter**.
/// 
/// TSn - Total Near for Supporters
/// TSst - Total stNear for Supporters
/// TKn - Total Near for Kickstarter
/// TKst - Total stNear for Kickstarter
/// TDst - Total Deposited in stNear
/// freeze - Price at freeze (near / stnear)
/// unfreeze - Price at unfreeze
/// 
/// (1) TSn  =  TDst * freeze;
/// (2) TSst =  TSn  / unfreeze;
/// (3) TSst =  TDst * (freeze / unfreeze);
/// (4) TKst =  TDst - TSst;
/// (5) TKst =  TDst * [1 - (freeze / unfreeze)];

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn kickstarter_withdraw(
        &mut self,
        kickstarter: &mut Kickstarter,
        price_at_unfreeze: Balance,
        receiver_id: AccountId,
    ) {
        let price_at_freeze = kickstarter.stnear_price_at_freeze.unwrap();
        let entity = WithdrawEntity::Kickstarter;
        let current_withdraw = kickstarter.get_stnear_withdraw(&entity);
        let interest = kickstarter.calculate_interest(price_at_freeze, price_at_unfreeze, current_withdraw);

        if interest > 0 {
            let new_withdraw = current_withdraw + interest;
            kickstarter.stnear_withdraw.insert(&entity, &new_withdraw);
            self.kickstarters.replace(kickstarter.id as u64, &kickstarter);

            nep141_token::ft_transfer(
                convert_to_valid_account_id(receiver_id),
                interest.into(),
                None,
                &self.metapool_contract_address,
                0,
                GAS_FOR_FT_TRANSFER
            ).then(
                ext_self_kickstarter::kickstarter_withdraw_resolve_transfer(
                    kickstarter.id.into(), 
                    interest.into(),
                    convert_to_valid_account_id(receiver_id),
                    &env::current_account_id(),
                    0,
                    env::prepaid_gas() - env::used_gas() - GAS_FOR_FT_TRANSFER
                )
            );
        } else {
            panic!("No more available interests for Kickstarter {}", kickstarter.id);
        }
    }

    #[private]
    pub fn kickstarter_withdraw_resolve_transfer(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
        receiver_id: ValidAccountId,
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "INTEREST WITHDRAW: {} stNEAR transfer to {}",
                    amount, receiver_id.to_string()
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} stNEAR of interest not transfered. Recovering Kickstarter {} state.",
                    amount, kickstarter_id
                );
                self.internal_restore_kickstarter_withdraw(amount, kickstarter_id.into())
            }
        }
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



    

    pub(crate) fn internal_restore_kickstarter_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId
    ){
        let mut kickstarter = self
        .kickstarters
        .get(kickstarter_id.into())
        .expect("kickstarter not found");
        // WARNING: next 2 line
        assert!(kickstarter.kickstarter_withdraw <= amount, "withdrawn amount is higher than expected");
        kickstarter.kickstarter_withdraw -= amount;
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }
}

impl Kickstarter {
    /// Function (5) from the Katherine math.
    fn calculate_interest(
        &self,
        price_at_freeze: Balance,
        price_at_unfreeze: Balance,
        current_withdraw: Balance
    ) -> Balance {
        assert!(price_at_unfreeze > price_at_freeze, "stNear price has not been updated, please wait!");
        let interest = self.total_deposited
            - proportional(
                self.total_deposited,
                price_at_freeze,
                price_at_unfreeze
            );
        interest - current_withdraw
    }
}
