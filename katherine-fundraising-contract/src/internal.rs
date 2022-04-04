use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId};

use crate::interface::*;

/*************/
/*  Asserts  */
/*************/

impl KatherineFundraising {
    pub(crate) fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }

    pub(crate) fn assert_unique_slug(&self, slug: &String) {
        assert!(
            self.kickstarter_id_by_slug.get(slug).is_none(),
            "Slug already exists. Choose a different one!"
        );
    }

    #[inline]
    pub(crate) fn assert_only_admin(&self) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "only allowed for admin"
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

    /// Inner method to get the given kickstarter.
    pub(crate) fn internal_get_kickstarter(&self, kickstarter_id: KickstarterId) -> Kickstarter {
        self.kickstarters
            .get(kickstarter_id as u64)
            .expect("Unknown KickstarterId")
    }

    /// Process a stNEAR deposit to Katherine Contract.
    pub(crate) fn internal_supporter_deposit(
        &mut self,
        supporter_id: &AccountId,
        amount: &Balance,
        kickstarter: &mut Kickstarter,
    ) {
        kickstarter.assert_within_funding_period();
        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        self.supporters.insert(&supporter_id, &supporter);
        kickstarter.total_deposited += amount;
        kickstarter.update_supporter_deposits(&supporter_id, amount);
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
    }

    /// Process a reward token deposit to Katherine Contract.
    pub(crate) fn internal_kickstarter_deposit(
        &mut self,
        amount: &Balance,
        kickstarter: &mut Kickstarter,
    ) {
        assert_eq!(
            &env::predecessor_account_id(),
            &kickstarter.token_contract_address,
            "Deposited tokens do not correspond to the Kickstarter contract."
        );
        assert!(
            get_current_epoch_millis() < kickstarter.open_timestamp,
            "Kickstarter Tokens should be provided before the funding period starts."
        );
        kickstarter.available_reward_tokens += amount;
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
    }

    /// Start the cross-contract call to activate the kickstarter.
    pub fn internal_activate_kickstarter(&mut self, kickstarter_id: KickstarterId, goal_id: GoalId) {
        log!("HERE WE ARE_4_test4");
        ext_self_metapool::get_st_near_price(
            //promise params
            &self.metapool_contract_address,
            0,
            GAS_FOR_GET_STNEAR,
        )
        .then(ext_self_kikstarter::activate_successful_kickstarter_after(
            kickstarter_id,
            goal_id,
            //promise params
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_GET_STNEAR,
        ));
    }

    // fn continues here after callback
    #[private]
    #[allow(unused)]
    pub(crate) fn activate_successful_kickstarter_after(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        goal_id: GoalIdJSON,
    ) {
        // NOTE: be careful on `#[callback]` here. If the get_stnear_price view call fails for some
        //    reason this call will not be entered, because #[callback] fails for failed_promises
        //    So *never* have something to rollback if the callback uses #[callback] params
        //    because the .after() will not be execute on error

        log!("START: after");
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        let st_near_price = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("Meta Pool is not available!"),
            PromiseResult::Successful(result) => {
                let price = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                Balance::from(price)
            },
        };
        log!("price: {}", st_near_price);

        log!("MIDDLE: here");
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let winning_goal = kickstarter.goals.get(goal_id as u64);
        match winning_goal {
            None => panic!("Kickstarter did not achieved any goal!"),
            Some(goal) => {
                assert!(
                    kickstarter.available_reward_tokens >= goal.tokens_to_release,
                    "Not enough available reward tokens to back the supporters rewards!"
                );
                // kickstarter.winner_goal_id = Some(goal.id);
                // kickstarter.active = false;
                // self.active_projects.remove(&kickstarter.id);
                // kickstarter.successful = Some(true);
                // kickstarter.set_katherine_fee(self.katherine_fee_percent, &goal);
                log!("END: here");
                kickstarter.stnear_price_at_freeze = Some(st_near_price.into());
                self.kickstarters
                    .replace(kickstarter_id as u64, &kickstarter);
            }
        }
    }

    pub(crate) fn internal_unfreeze_kickstarter_funds(&mut self, kickstarter_id: KickstarterId) {
        ext_self_metapool::get_st_near_price(
            //promise params
            &self.metapool_contract_address,
            NO_DEPOSIT,
            GAS_FOR_GET_STNEAR,
        )
        .then(ext_self_kikstarter::set_stnear_price_at_unfreeze(
            kickstarter_id,
            //promise params
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() - env::used_gas() - GAS_FOR_GET_STNEAR,
        ));
    }

    // fn continues here after callback
    #[private]
    #[allow(unused)]
    pub(crate) fn set_stnear_price_at_unfreeze(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        #[callback] st_near_price: U128,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.stnear_price_at_unfreeze = Some(st_near_price.into());
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
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
        let available_rewards =
            get_linear_release_proportion(total_rewards, cliff_timestamp, end_timestamp);
        if available_rewards == 0 {
            return 0;
        };
        let deposit = kickstarter
            .deposits
            .get(&supporter_id)
            .expect("deposit not found");
        proportional(deposit, available_rewards, kickstarter.total_deposited)
    }

    pub(crate) fn internal_withdraw_kickstarter_tokens(
        &mut self,
        requested_amount: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId,
    ) {
        let goal = kickstarter.get_winner_goal();
        assert_eq!(
            kickstarter.successful,
            Some(true),
            "kickstarter has not reached any goal"
        );
        assert!(
            goal.cliff_timestamp < get_current_epoch_millis(),
            "tokens have not been released yet"
        );

        let total_supporter_rewards =
            self.internal_get_supporter_total_rewards(&supporter_id, &kickstarter, goal);
        assert!(
            total_supporter_rewards >= 1,
            "less than one token to withdraw"
        );
        assert!(
            requested_amount <= total_supporter_rewards,
            "not enough tokens, available balance is {}",
            total_supporter_rewards
        );

        let mut supporter_withdraw: Balance = match kickstarter.withdraw.get(&supporter_id) {
            Some(value) => value,
            None => 0,
        };
        supporter_withdraw += requested_amount;
        kickstarter
            .withdraw
            .insert(&supporter_id, &supporter_withdraw);
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
    }
    pub(crate) fn internal_restore_kickstarter_excedent_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
    ) {
        let mut kickstarter = self
            .kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarter not found");

        kickstarter.available_reward_tokens += amount;
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
    }

    pub(crate) fn internal_restore_kickstarter_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId,
    ) {
        let mut kickstarter = self
            .kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarter not found");
        let mut withdraw = kickstarter.withdraw.get(&supporter_id).unwrap_or_default();

        assert!(withdraw >= amount, "withdrawn amount too high");

        if withdraw == amount {
            kickstarter.withdraw.remove(&supporter_id);
        } else {
            withdraw -= amount;
            kickstarter.withdraw.insert(&supporter_id, &withdraw);
        }
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
    }

    /// This function is for the Supporter withdrawal of stNear tokens. The kickstarter.total_deposited
    /// is only modified during the funding period. After the project evaluation, the value is kept only
    /// as a reference.
    pub(crate) fn internal_supporter_withdraw(
        &mut self,
        requested_amount: Balance,
        deposit: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId
    ) {
        assert!(requested_amount <= deposit, "withdraw amount exceeds balance");
        if deposit == requested_amount{
            kickstarter.deposits.remove(&supporter_id);
        }
        else{
            let new_total = deposit - requested_amount; 
            kickstarter.deposits.insert(&supporter_id, &new_total);
        }
        if kickstarter.is_within_funding_period() {
            kickstarter.total_deposited -= requested_amount;
        }
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    } 

    pub(crate) fn internal_restore_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId,
    ) {
        let mut kickstarter = self
            .kickstarters
            .get(kickstarter_id as u64)
            .expect("kickstarter not found");
        let mut deposit = kickstarter.deposits.get(&supporter_id).unwrap_or_default();

        deposit += amount;
        kickstarter.deposits.insert(&supporter_id, &deposit);
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
    }
}
