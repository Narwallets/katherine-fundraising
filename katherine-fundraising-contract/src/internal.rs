use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen};

use crate::interface::*;

impl KatherineFundraising {
    #[inline]
    pub(crate) fn assert_only_admin(&self) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "only allowed for admin"
        );
    }
}

/*************************/
/*  pToken Calculations  */
/*************************/

impl KatherineFundraising {
    pub(crate) fn calculate_total_tokens_to_release(
        &self,
        kickstarter: &Kickstarter,
        tokens_to_release_per_stnear: Balance
    ) -> Balance {
        proportional(
            kickstarter.total_deposited,
            tokens_to_release_per_stnear,
            NEAR
        )
    }

    pub(crate) fn calculate_katherine_fee(
        &self,
        total_tokens_to_release: Balance
    ) -> Balance {
        proportional(
            self.katherine_fee_percent as u128,
            total_tokens_to_release,
            BASIS_POINTS
        )
    }

    pub(crate) fn calculate_max_tokens_to_release(
        &self,
        kickstarter: &Kickstarter,
    ) -> Balance {
        proportional(
            kickstarter.deposits_hard_cap,
            kickstarter.max_tokens_to_release_per_stnear,
            NEAR
        ) 
    }
}

/**********************/
/*  Internal methods  */
/**********************/

#[near_bindgen]
impl KatherineFundraising {
    /// Inner method to get the given supporter or a new default value supporter.
    pub(crate) fn internal_get_supporter(&self, supporter_id: &SupporterId) -> Supporter {
        self.supporters.get(supporter_id).unwrap_or(Supporter::new(supporter_id))
    }

    /// Inner method to get the given kickstarter.
    pub(crate) fn internal_get_kickstarter(&self, kickstarter_id: KickstarterId) -> Kickstarter {
        self.kickstarters
            .get(kickstarter_id as u64)
            .expect("Unknown KickstarterId")
    }

    pub(crate) fn activate_successful_kickstarter(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        goal_id: GoalIdJSON,
    ) {
        ext_self_metapool::get_st_near_price(
            //promise params
            &self.metapool_contract_address,
            0,
            GAS_FOR_GET_STNEAR,
        ).then(
            ext_self_kickstarter::activate_successful_kickstarter_after(
                kickstarter_id,
                goal_id,
                //promise params
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_GET_STNEAR
            )
        );
    }

    // fn continues here after callback
    #[private]
    pub fn activate_successful_kickstarter_after(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        goal_id: GoalIdJSON,
    ) {
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
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        match kickstarter.goals.get(goal_id as u64) {
            None => panic!("Kickstarter did not achieved any goal!"),
            Some(goal) => {
                let total_tokens_to_release = self.calculate_total_tokens_to_release(
                    &kickstarter,
                    goal.tokens_to_release_per_stnear
                );
                let katherine_fee = self.calculate_katherine_fee(total_tokens_to_release);
                assert!(
                    kickstarter.available_reward_tokens >= (total_tokens_to_release + katherine_fee),
                    "Not enough available reward tokens to back the supporters rewards!"
                );
                kickstarter.winner_goal_id = Some(goal.id);
                kickstarter.active = false;
                self.active_projects.remove(&kickstarter.id);
                kickstarter.successful = Some(true);
                kickstarter.katherine_fee = Some(katherine_fee);
                kickstarter.total_tokens_to_release = Some(total_tokens_to_release);
                kickstarter.stnear_price_at_freeze = Some(st_near_price.into());
                self.kickstarters
                    .replace(kickstarter_id as u64, &kickstarter);
            }
        }
    }

    pub(crate) fn internal_unfreeze_kickstarter_funds(
        &mut self,
        kickstarter_id: KickstarterId
    ) {
        ext_self_metapool::get_st_near_price(
            &self.metapool_contract_address,
            NO_DEPOSIT,
            GAS_FOR_GET_STNEAR
        ).then(
            ext_self_kickstarter::set_stnear_price_at_unfreeze(
                kickstarter_id,
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_GET_STNEAR
            )
        );
    }

    // fn continues here after callback
    #[private]
    pub fn set_stnear_price_at_unfreeze(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
    ) {
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
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.stnear_price_at_unfreeze = Some(st_near_price.into());
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
    }

    /// This is the amount of rewards that the supporter could claim regardless of the current timestamp.
    pub(crate) fn internal_get_supporter_rewards(
        &self,
        supporter_id: &SupporterId,
        kickstarter: &Kickstarter,
        tokens_to_release_per_stnear: Balance,
    ) -> Balance {
        let deposit = kickstarter.get_deposit(&supporter_id);
        proportional(deposit, tokens_to_release_per_stnear, NEAR)
            - kickstarter.get_rewards_withdraw(&supporter_id)
    }

    pub(crate) fn internal_get_available_rewards(
        &self,
        supporter_id: &SupporterId,
        kickstarter: &Kickstarter,
    ) -> Balance {
        let goal = kickstarter.get_winner_goal();
        let deposit = kickstarter.get_deposit(&supporter_id);

        let total_supporter_rewards = proportional(
            deposit,
            goal.tokens_to_release_per_stnear,
            NEAR
        );
        get_linear_release_proportion(
            total_supporter_rewards,
            goal.cliff_timestamp,
            goal.end_timestamp
        ).saturating_sub(kickstarter.get_rewards_withdraw(&supporter_id))
    }
}
