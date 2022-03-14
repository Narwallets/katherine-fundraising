use crate::*;
use near_sdk::{near_bindgen, log, AccountId};
use near_sdk::serde_json::{json};

use crate::{types::*, errors::*, utils::*};

impl KatherineFundraising {
    pub fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }
}

/**********************/
/*  Internal methods  */
/**********************/
#[near_bindgen]
impl KatherineFundraising {

    /// Inner method to get the given supporter or a new default value supporter.
    pub(crate) fn internal_get_supporter(&self, supporter_id: &SupporterId) -> Supporter {
        self.supporters.get(supporter_id).unwrap_or_default()
    }

    /// Inner method to save the given supporter for a given supporter ID.
    /// If the supporter balances are 0, the supporter is deleted instead to release storage.
    pub(crate) fn internal_update_supporter(&mut self, supporter_id: &SupporterId, supporter: &Supporter) {
        if supporter.is_empty() {
            self.supporters.remove(supporter_id);
        } else {
            self.supporters.insert(supporter_id, &supporter); //insert_or_update
        }
    }

    /// Inner method to get the given kickstarter.
    pub(crate) fn internal_get_kickstarter(&self, kickstarter_id: KickstarterId) -> Kickstarter {
        self.kickstarters.get(kickstarter_id as u64).expect("Unknown kickstarter id")
    }

    /// Process a stNEAR deposit to Katherine Contract.
    pub(crate) fn internal_supporter_deposit(
        &mut self,
        supporter_id: &AccountId,
        amount: &Balance,
        kickstarter: &mut Kickstarter
    ) -> Result<Balance, String> {
        let current_timestamp = env::block_timestamp();
        if current_timestamp >= kickstarter.close_timestamp || current_timestamp < kickstarter.open_timestamp {
            return Err("Not within the funding period.".into());
        }

        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        self.supporters.insert(&supporter_id, &supporter);
        kickstarter.total_deposited += amount;
        kickstarter.update_supporter_deposits(&supporter_id, amount);

        // Return unused amount.
        Ok(0)
    }

    /// Process a reward token deposit to Katherine Contract.
    pub(crate) fn internal_kickstarter_deposit(
        &mut self,
        amount: &Balance,
        kickstarter: &mut Kickstarter    
    ) -> Result<Balance, String> {
        assert_eq!(
            &env::predecessor_account_id(),
            &kickstarter.token_contract_address,
            "Deposited tokens do not correspond to the Kickstarter contract."
        );

        let current_timestamp = env::block_timestamp();
        if current_timestamp > kickstarter.open_timestamp {
            return Err("Kickstarter Tokens should be provided before the funding period starts.".into());
        }
        kickstarter.available_reward_tokens += amount;

        // Return unused amount.
        Ok(0)
    }

    /// Start the cross-contract call to activate the kickstarter.
    pub(crate) fn internal_activate_kickstarter(&mut self, kickstarter_id: KickstarterId) {
        ext_metapool::get_st_near_price(
            //promise params
            &self.metapool_contract_address,
            NO_DEPOSIT,
            GAS_FOR_GET_STNEAR,
        )
        .then(ext_self::activate_successful_kickstarter_after(
            kickstarter_id,
            //promise params
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_GET_STNEAR,
        ));
    }

    // fn continues here after callback
    #[private]
    pub(crate) fn activate_successful_kickstarter_after(
        &mut self,
        kickstarter_id: KickstarterIdJSON, 
        #[callback] st_near_price: U128String,
    ) {
        // NOTE: be careful on `#[callback]` here. If the get_stnear_price view call fails for some
        //    reason this call will not be entered, because #[callback] fails for failed_promises
        //    So *never* have something to rollback if the callback uses #[callback] params
        //    because the .after() will not be execute on error 

        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let winning_goal = kickstarter.get_achieved_goal();
        match winning_goal {
            None => panic!("Kickstarter did not achieved any goal!"),
            Some(goal) => {
                assert!(
                    kickstarter.available_reward_tokens >= goal.tokens_to_release,
                    "Not enough available reward tokens to back the supporters rewards!"
                );
                kickstarter.winner_goal_id = Some(goal.id);
                kickstarter.active = false;
                kickstarter.successful = Some(true);
                kickstarter.set_katherine_fee(self.katherine_fee_percent, &goal);
                kickstarter.stnear_value_in_near = Some(st_near_price.into());
                self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
            }
        }
    }

    pub(crate) fn internal_verify_total_deposited(
        &self,
        kickstarter: &Kickstarter,
        supporter_id: &SupporterId,
        total_deposited: Balance
    ) -> bool {
        match kickstarter.deposits.get(&supporter_id) {
            Some(amount) => return amount == total_deposited,
            None => return false,
        }
    }

    pub(crate) fn internal_withdraw_kickstarter_tokens(
        &mut self,
        requested_amount: Balance,
        kickstarter: &Kickstarter,
        supporter_id: &AccountId
    ){
        assert!(kickstarter.successful == Some(true), "kickstarter has not reached any goal");
        let goal = kickstarter.get_goal();
        let cliff_timestamp = u128::from(goal.cliff_timestamp);
        assert!(goal.cliff_timestamp < get_epoch_millis(), "tokens have not been released yet");
        let mut available_balance = (((u128::from(get_epoch_millis()) - cliff_timestamp) * kickstarter.get_total_rewards_for_supporters()) / (u128::from(goal.end_timestamp) - cliff_timestamp));
        assert!(available_balance >= 1, "less than one token to withdraw");

        if available_balance > kickstarter.get_total_rewards_for_supporters() {
            available_balance = kickstarter.get_total_rewards_for_supporters();
        }
        assert!(requested_amount <= available_balance, "not enough tokens, available balance is {}", available_balance);
        let mut deposit = kickstarter.deposits.get(&supporter_id).expect("deposit not found");
        let supporter_available = deposit * available_balance / kickstarter.total_deposited;
        let mut suporter_withdraw = kickstarter.withdraw.get(&supporter_id).unwrap_or_default();
        suporter_withdraw += requested_amount;
        kickstarter.withdraw.insert(&supporter_id, &suporter_withdraw);
    }

    pub(crate) fn internal_restore_kickstarter_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId
    ){
        let mut kickstarter = self.kickstarters
        .get(kickstarter_id as u64)
        .expect("kickstarted not found");
        let mut withdraw = kickstarter.withdraw.get(&supporter_id).unwrap_or_default();

        withdraw -= amount;
        assert!(withdraw >= 0, "withdrawn amount too high");

        if withdraw == 0 {
            kickstarter.withdraw.remove(&supporter_id);
        }
        else{
            kickstarter.withdraw.insert(&supporter_id, &withdraw);
        }
    }

    #[inline]
    pub(crate) fn only_admin(&self, account: AccountId){
        assert!(env::predecessor_account_id() == self.owner_id, "only allowed for admin");
    }



    pub(crate) fn internal_withdraw(
        &mut self,
        requested_amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: &AccountId
    ) {
        let mut kickstarter = self.kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarted not found");
        assert!(
            kickstarter.successful != Some(true) &&
            kickstarter.vesting_timestamp >= get_epoch_millis()
            , "can not withdraw from successfull kickstarter before vesting period ends"
        );

        let mut deposit = kickstarter.deposits.get(&supporter_id).expect("deposit not found");

        assert!(requested_amount <= deposit, "withdraw amount exceeds balance");
        if deposit == requested_amount{
            kickstarter.deposits.remove(&supporter_id);
        }
        else{
            deposit -= requested_amount;
            kickstarter.deposits.insert(&supporter_id, &deposit);
        }
        //UPG check if it should refund freed storage
    }

    pub(crate) fn internal_kickstarter_withdraw(&mut self, kickstarter: &Kickstarter){
        unimplemented!();
    }

    pub(crate) fn internal_restore_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId
    ) {
        let mut kickstarter = self.kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarted not found");
        let mut deposit = kickstarter.deposits.get(&supporter_id).unwrap_or_default();

        deposit += amount;
        kickstarter.deposits.insert(&supporter_id, &deposit);
    }
}
