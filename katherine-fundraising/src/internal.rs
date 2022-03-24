use near_sdk::{near_bindgen, AccountId};

use crate::*;

/********************/
/*  Assert methods  */
/********************/

impl KatherineFundraising {
    pub fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }

    pub fn assert_unique_slug(&self, slug: &String) {
        assert!(
            self.kickstarter_id_by_slug.get(slug).is_none(),
            "Slug already exists. Choose a different one!"
        );
    }

    #[inline]
    pub(crate) fn assert_only_admin(&self){
        assert!(env::predecessor_account_id() == self.owner_id, "only allowed for admin");
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

    /// Inner method to get the given kickstarter.
    pub(crate) fn internal_get_kickstarter(&self, kickstarter_id: KickstarterId) -> Kickstarter {
        self.kickstarters.get(kickstarter_id as u64).expect("Unknown KickstarterId")
    }

    /// Process a stNEAR deposit to Katherine Contract.
    pub(crate) fn internal_supporter_deposit(
        &mut self,
        supporter_id: &AccountId,
        amount: &Balance,
        kickstarter_id: &KickstarterId
    ) -> Result<Balance, String> {
        let mut kickstarter: Kickstarter = self.internal_get_kickstarter(*kickstarter_id);
        let current_timestamp = get_current_epoch_millis();
        if current_timestamp >= kickstarter.close_timestamp || current_timestamp < kickstarter.open_timestamp {
            panic!("Not within the funding period.");
        }

        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        self.supporters.insert(supporter_id, &supporter);
        kickstarter.total_deposited += amount;
        let mut deposit = kickstarter.deposits.get(supporter_id).unwrap_or_default();
        deposit += amount;
        kickstarter.deposits.insert(supporter_id, &deposit);
        self.kickstarters.replace(*kickstarter_id as u64, &kickstarter);
        // Return unused amount.
        Ok(0)
    }

    /// Process a reward token deposit to Katherine Contract.
    pub(crate) fn internal_kickstarter_deposit(
        &mut self,
        amount: &Balance,
        kickstarter_id: &KickstarterId    
    ) -> Result<Balance, String> {
        let mut kickstarter: Kickstarter = self.internal_get_kickstarter(*kickstarter_id);
        assert_eq!(
            &env::predecessor_account_id(),
            &kickstarter.token_contract_address,
            "Deposited tokens do not correspond to the Kickstarter contract."
        );

        let current_timestamp = get_current_epoch_millis();
        if current_timestamp > kickstarter.open_timestamp {
            panic!("Kickstarter Tokens should be provided before the funding period starts.");
        }
        kickstarter.available_reward_tokens += amount;
        self.kickstarters.replace(*kickstarter_id as u64, &kickstarter);
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

    pub(crate) fn internal_get_supporter_total_rewards(
        &self,
        supporter_id: &SupporterId,
        kickstarter: &Kickstarter,
        goal: Goal,
    ) -> Balance {
        let cliff_timestamp = goal.cliff_timestamp;
        let end_timestamp = goal.end_timestamp;
        let total_rewards = kickstarter.get_total_rewards_for_supporters();
        let available_rewards = get_linear_release_proportion(total_rewards, cliff_timestamp, end_timestamp);
        if available_rewards == 0 {return 0};
        let deposit = kickstarter.deposits.get(&supporter_id).expect("deposit not found");
        proportional(
            deposit,
            available_rewards,
            kickstarter.total_deposited
        )
    }

    pub(crate) fn internal_withdraw_kickstarter_tokens(
        &mut self,
        requested_amount: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId,
    ){
        let goal = kickstarter.get_goal();
        assert_eq!(kickstarter.successful, Some(true), "kickstarter has not reached any goal");
        assert!(goal.cliff_timestamp < get_current_epoch_millis(), "tokens have not been released yet");

        let total_supporter_rewards = self.internal_get_supporter_total_rewards(&supporter_id, &kickstarter, goal);
        assert!(total_supporter_rewards >= 1, "less than one token to withdraw");
        assert!(requested_amount <= total_supporter_rewards, "not enough tokens, available balance is {}", total_supporter_rewards);

        let mut supporter_withdraw: Balance = match kickstarter.withdraw.get(&supporter_id) {
            Some(value) => value,
            None => 0,
        };
        supporter_withdraw += requested_amount;
        kickstarter.withdraw.insert(&supporter_id, &supporter_withdraw);
    }
    
    pub(crate) fn internal_restore_kickstarter_excedent_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
    ){
        let mut kickstarter = self.kickstarters
        .get(kickstarter_id as u64)
        .expect("kickstarter not found");

        kickstarter.available_reward_tokens += amount;
    }

    pub(crate) fn internal_restore_kickstarter_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId
    ){
        let mut kickstarter = self.kickstarters
        .get(kickstarter_id as u64)
        .expect("kickstarter not found");
        let mut withdraw = kickstarter.withdraw.get(&supporter_id).unwrap_or_default();

        assert!(withdraw >= amount, "withdrawn amount too high");
        withdraw -= amount;

        if withdraw == 0 {
            kickstarter.withdraw.remove(&supporter_id);
        }
        else{
            kickstarter.withdraw.insert(&supporter_id, &withdraw);
        }
    }

    pub(crate) fn internal_withdraw(
        &mut self,
        requested_amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: &AccountId
    )-> Balance {
        let mut kickstarter = self.kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarter not found");

        let mut deposit = kickstarter.deposits.get(&supporter_id).expect("deposit not found");
        let amount_to_remove = if kickstarter.successful == Some(true) {
            assert!(
                kickstarter.get_goal().end_timestamp >= get_current_epoch_millis()
                , "can not withdraw from successfull kickstarter before vesting period ends"
            );
            kickstarter.assert_unfreezed_funds();
            let price_at_freeze = kickstarter.stnear_price_at_freeze.expect("Price at freeze is not defined!");
            let price_at_unfreeze = kickstarter.stnear_price_at_unfreeze.expect("Price at unfreeze is not defined. Please unfreeze kickstarter funds with fn: unfreeze_kickstarter_funds!");
            let max_amount_to_withdraw = proportional(deposit, price_at_freeze, price_at_unfreeze);
            assert!(requested_amount <= max_amount_to_withdraw, "Amount not available!");
            if is_close(requested_amount, max_amount_to_withdraw) {
                max_amount_to_withdraw
            } else {
                requested_amount
            }
        } else {
            assert!(requested_amount <= deposit, "Amount not available!");
            if is_close(requested_amount, deposit) {
                deposit
            } else {
                requested_amount
            }
        };

        if deposit == amount_to_remove{
            kickstarter.deposits.remove(&supporter_id);
        }
        else{
            deposit -= amount_to_remove;
            kickstarter.deposits.insert(&supporter_id, &deposit);
        }
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
        //UPG check if it should refund freed storage
        amount_to_remove
    }

    pub(crate) fn internal_restore_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId
    ) {
        let mut kickstarter = self.kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarter not found");
        let mut deposit = kickstarter.deposits.get(&supporter_id).unwrap_or_default();

        deposit += amount;
        kickstarter.deposits.insert(&supporter_id, &deposit);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
    }


    // pub(crate) fn internal_kickstarter_withdraw(amount: Balance, kickstarter: &mut Kickstarter) {
    //     kickstarter.total_deposited -= amount;
    //     self.kickstarters.insert(kickstarter.id, kickstarter);
    // }

    // pub(crate) fn internal_restore_kickstarter_withdraw(amount: Balance, kickstarter: KickstarterId)-> Balance{

    // }
}
