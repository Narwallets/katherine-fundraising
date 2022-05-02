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

/************************/
/*  Supporter Withdraw  */
/************************/

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn internal_supporter_withdraw_before_freeze(
        &mut self,
        requested_amount: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: SupporterId
    ) {
        let deposit = kickstarter.get_deposit(&supporter_id);
        assert!(requested_amount <= deposit, "Not available amount!");
        let amount_to_withdraw = if is_close(requested_amount, deposit) {
            deposit
        } else {
            requested_amount
        };

        self.supporter_withdraw_before_freeze(
            amount_to_withdraw,
            deposit,
            kickstarter,
            &supporter_id
        );
        let supporter_id = convert_to_valid_account_id(supporter_id);
        nep141_token::ft_transfer(
            supporter_id.clone(),
            BalanceJSON::from(amount_to_withdraw),
            None,
            &self.metapool_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        ).then(
            ext_self_metapool::return_tokens_before_freeze_callback(
                supporter_id,
                kickstarter.id.into(),
                BalanceJSON::from(amount_to_withdraw),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER
            )
        );
    }

    /// This function is for the Supporter withdrawal of stNear tokens. The kickstarter.total_deposited
    /// is only modified during the funding period. After the project evaluation, the value is kept only
    /// as a reference. NO REWARDS ACHIEVED!
    pub(crate) fn supporter_withdraw_before_freeze(
        &mut self,
        requested_amount: Balance,
        deposit: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId
    ) {
        if deposit == requested_amount {
            kickstarter.deposits.remove(&supporter_id);
            // Remove Kickstarter from the supported projects.
            // This is possible because no Rewards can be claimed.
            let mut supporter = self.internal_get_supporter(&supporter_id);
            supporter.supported_projects.remove(&kickstarter.id);
            self.supporters.insert(&supporter_id, &supporter);
        } else {
            let new_total = deposit - requested_amount;
            kickstarter.deposits.insert(&supporter_id, &new_total);
        }
        if kickstarter.is_within_funding_period() {
            kickstarter.total_deposited -= requested_amount;
        }
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }

    #[private]
    pub fn return_tokens_before_freeze_callback(
        &mut self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
        amount: BalanceJSON
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                // TODO: FREE THE KICKSTARTER FROM THE SUPPORTER IF supporter.is_supporting(kickstarter) == False
                log!("token transfer {}", amount);
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount
                );
                self.internal_restore_withdraw_before_freeze(
                    amount,
                    kickstarter_id,
                    supporter_id.to_string()
                );
            },
        };
    }

    fn internal_restore_withdraw_before_freeze(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let deposit = match kickstarter.deposits.get(&supporter_id) {
            None => {
                // If the deposit was deleted, then restore the supported project too.
                let mut supporter = self.internal_get_supporter(&supporter_id);
                supporter.supported_projects.insert(&kickstarter.id);
                self.supporters.insert(&supporter_id, &supporter);
                amount
            },
            Some(balance) => balance + amount,
        }; 
        kickstarter.deposits.insert(&supporter_id, &deposit);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
    }

    pub(crate) fn internal_supporter_withdraw_after_unfreeze(
        &mut self,
        requested_amount: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: SupporterId
    ) {
        let entity = WithdrawEntity::Supporter(supporter_id.to_string());
        let available_to_withdraw = kickstarter.get_after_unfreeze_deposits(&supporter_id)
            - kickstarter.get_stnear_withdraw(&entity);
        assert!(requested_amount <= available_to_withdraw, "Not available amount!");
        let amount_to_withdraw = if is_close(requested_amount, available_to_withdraw) {
            available_to_withdraw
        } else {
            requested_amount
        };

        self.supporter_withdraw_after_unfreeze(
            amount_to_withdraw,
            available_to_withdraw,
            kickstarter,
            &supporter_id
        );
        let supporter_id = convert_to_valid_account_id(supporter_id);
        nep141_token::ft_transfer(
            supporter_id.clone(),
            BalanceJSON::from(amount_to_withdraw),
            None,
            &self.metapool_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        ).then(
            ext_self_metapool::return_tokens_after_unfreeze_callback(
                supporter_id,
                kickstarter.id.into(),
                BalanceJSON::from(amount_to_withdraw),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER
            )
        );
    }

    /// This function is for the Supporter withdrawal of stNear tokens after the unfreeze.
    /// REWARDS ACHIEVED for all Supporters!
    pub(crate) fn supporter_withdraw_after_unfreeze(
        &mut self,
        requested_amount: Balance,
        available_to_withdraw: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId
    ) {
        let entity = WithdrawEntity::Supporter(supporter_id.to_string());
        let current_withdraw = kickstarter.get_stnear_withdraw(&entity);
        let new_withdraw = current_withdraw + requested_amount;
        kickstarter.stnear_withdraw.insert(&entity, &new_withdraw);
        if available_to_withdraw == requested_amount {
            // Do not remove from supported projects unless no more rewards.
            let rewards = self.internal_get_supporter_rewards(
                &supporter_id,
                &kickstarter,
                kickstarter.get_winner_goal().tokens_to_release_per_stnear
            );
            if rewards == 0 {
                let mut supporter = self.internal_get_supporter(&supporter_id);
                supporter.supported_projects.remove(&kickstarter.id);
                self.supporters.insert(&supporter_id, &supporter);
            }
        }
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }

    #[private]
    pub fn return_tokens_after_unfreeze_callback(
        &mut self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
        amount: BalanceJSON
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                // TODO: FREE THE KICKSTARTER FROM THE SUPPORTER IF supporter.is_supporting(kickstarter) == False
                log!("token transfer {}", amount);
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount
                );
                self.internal_restore_withdraw_after_unfreeze(
                    amount,
                    kickstarter_id,
                    supporter_id.to_string()
                );
            },
        };
    }

    pub(crate) fn internal_restore_withdraw_after_unfreeze(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: AccountId,
    ) {
        let entity = WithdrawEntity::Supporter(supporter_id.to_string());
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let deposit = kickstarter.get_after_unfreeze_deposits(&supporter_id);
        let supporter_withdraw = kickstarter.get_stnear_withdraw(&entity);
        let new_withdraw = supporter_withdraw - amount;

        kickstarter.stnear_withdraw.insert(&entity, &new_withdraw);
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);

        // If the withdraw fail, add the Kickstarter back to the supported projects.
        if supporter_withdraw == deposit {
            let mut supporter = self.internal_get_supporter(&supporter_id);
            supporter.supported_projects.insert(&kickstarter.id);
            self.supporters.insert(&supporter_id, &supporter);
        }
    }
}

/*********************/
/*  Supporter Claim  */
/*********************/

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn internal_claim_kickstarter_tokens(
        &mut self,
        requested_amount: BalanceJSON,
        kickstarter: &mut Kickstarter,
        supporter_id: SupporterId,
    ) {
        let kickstarter_id = kickstarter.id;
        self.update_supporter_claims(
            Balance::from(requested_amount),
            kickstarter,
            &supporter_id
        );

        nep141_token::ft_transfer(
            convert_to_valid_account_id(supporter_id.clone()),
            requested_amount,
            None,
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        ).then(
            ext_self_kickstarter::return_tokens_from_kickstarter_callback(
                convert_to_valid_account_id(supporter_id.clone()),
                kickstarter_id,
                requested_amount,
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER
            )
        );
    }

    pub(crate) fn update_supporter_claims(
        &mut self,
        requested_amount: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId,
    ) {
        assert!(requested_amount > 0, "Supporter does not have available Kickstarter Tokens");
        let goal = kickstarter.get_winner_goal();
        assert_eq!(
            kickstarter.successful, Some(true),
            "Kickstarter was unsuccessful."
        );
        assert!(
            goal.cliff_timestamp < get_current_epoch_millis(),
            "Tokens not released."
        );

        let rewards = self.internal_get_available_rewards(&supporter_id, &kickstarter);
        assert!(
            requested_amount <= rewards,
            "Not enough tokens, available balance is {}.",
            rewards
        );

        let mut withdraw: Balance = kickstarter.get_rewards_withdraw(&supporter_id);
        withdraw += requested_amount;
        kickstarter.rewards_withdraw.insert(&supporter_id, &withdraw);
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }

    #[private]
    pub fn return_tokens_from_kickstarter_callback(
        &mut self,
        account_id: AccountId,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "CLAIM WITHDRAW: {} pTOKEN transfered to Supporter {}",
                    amount, account_id
                );
            }
            PromiseResult::Failed => {
                log!(
                    "Token transfer failed {}. recovering account state",
                    amount
                );
                self.internal_restore_supporter_withdraw_from_kickstarter(
                    amount,
                    kickstarter_id,
                    account_id,
                )
            }
        }
    }

    pub(crate) fn internal_restore_supporter_withdraw_from_kickstarter(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
        supporter_id: SupporterId,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let mut withdraw = kickstarter.get_rewards_withdraw(&supporter_id);
        assert!(withdraw >= amount, "Withdraw amount too high.");

        if withdraw == amount {
            kickstarter.rewards_withdraw.remove(&supporter_id);
        } else {
            withdraw -= amount;
            kickstarter.rewards_withdraw.insert(&supporter_id, &withdraw);
        }
        self.kickstarters.replace(kickstarter_id as u64, &kickstarter);
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

    /// Process a stNEAR deposit to Katherine Contract.
    pub(crate) fn internal_supporter_deposit(
        &mut self,
        supporter_id: &AccountId,
        amount: &Balance,
        kickstarter: &mut Kickstarter,
    ) {
        // Update Kickstarter
        kickstarter.assert_within_funding_period();
        kickstarter.assert_enough_reward_tokens();

        let new_total_deposited = kickstarter.total_deposited + amount;
        assert!(
            new_total_deposited <= kickstarter.deposits_hard_cap,
            "The deposits hard cap cannot be exceeded!"
        );
        kickstarter.total_deposited = new_total_deposited;
        kickstarter.update_supporter_deposits(&supporter_id, amount);
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);

        // Update Supporter.
        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        supporter.supported_projects.insert(&kickstarter.id);
        self.supporters.insert(&supporter_id, &supporter);
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
            get_current_epoch_millis() < kickstarter.close_timestamp,
            "Kickstarter Tokens should be provided before the funding period ends."
        );
        let max_tokens_to_release = self.calculate_max_tokens_to_release(&kickstarter);
        let min_tokens_to_allow_support = max_tokens_to_release
            + self.calculate_katherine_fee(max_tokens_to_release);
        kickstarter.available_reward_tokens += amount;
        kickstarter.enough_reward_tokens = {
            kickstarter.available_reward_tokens >= min_tokens_to_allow_support
        };
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
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
        let total_supporter_rewards = self.internal_get_supporter_rewards(
            &supporter_id,
            &kickstarter,
            goal.tokens_to_release_per_stnear,
        );
        get_linear_release_proportion(
            total_supporter_rewards,
            goal.cliff_timestamp,
            goal.end_timestamp
        )
    }

    pub(crate) fn internal_withdraw_excedent(&mut self, kickstarter: &Kickstarter, excedent: Balance) {
        nep141_token::ft_transfer(
            convert_to_valid_account_id(env::predecessor_account_id()),
            excedent.into(),
            Some("withdraw excedent from kickstarter".to_string()),
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        ).then(
            ext_self_kickstarter::kickstarter_withdraw_excedent_callback(
                kickstarter.id,
                excedent.into(),
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER,
            ),
        );
    }

    #[private]
    pub fn kickstarter_withdraw_excedent_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", amount);
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount
                );
                self.internal_restore_kickstarter_excedent_withdraw(
                    amount,
                    kickstarter_id,
                )
            }
        }
    }

    fn internal_restore_kickstarter_excedent_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.available_reward_tokens += amount;
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
    }

    pub(crate) fn internal_withdraw_katherine_fee(&mut self, kickstarter: &mut Kickstarter, katherine_fee: Balance) {
        kickstarter.katherine_fee = Some(0);
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
        let katherine_fee = U128::from(katherine_fee);
        nep141_token::ft_transfer(
            convert_to_valid_account_id(env::predecessor_account_id()),
            katherine_fee,
            None,
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER
        ).then(
            ext_self_kickstarter::withdraw_kickstarter_fee_callback(
                kickstarter.id,
                katherine_fee,
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER
            )
        );
    }

    #[private]
    pub fn withdraw_kickstarter_fee_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} pTOKEN withdraw from KickstarterId {} to Account {}",
                    amount,
                    kickstarter_id,
                    env::predecessor_account_id(),
                );
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount
                );
                let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
                kickstarter.katherine_fee = Some(amount);
                self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
            }
        }
    }
}
