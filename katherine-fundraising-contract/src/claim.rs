use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen};

use crate::interface::*;

/// Claim is only for the **project tokens** in Katherine.

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

    fn update_supporter_claims(
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
        let amount_to_withdraw = if is_close(requested_amount, rewards) {
            rewards
        } else {
            requested_amount
        };

        // For supporters, Kath must track claims and withdraw in order to remove the supporter
        // when both are zero.
        if kickstarter.is_unfreeze() {
            self.remove_from_supported_claim(
                amount_to_withdraw,
                kickstarter,
                supporter_id,
                goal,
            );
        }

        let current_withdraw = kickstarter.get_rewards_withdraw(&supporter_id);
        let new_withdraw = current_withdraw + amount_to_withdraw;
        kickstarter.rewards_withdraw.insert(&supporter_id, &new_withdraw);
        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }

    /// The contrapart in the withdraw for this function is **remove_from_supported_withdraw**.
    fn remove_from_supported_claim(
        &mut self,
        amount_to_withdraw: Balance,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId,
        goal: Goal,
    ) {
        let entity = WithdrawEntity::Supporter(supporter_id.to_string());
        let available_stnear = kickstarter.get_after_unfreeze_deposits(&supporter_id)
            - kickstarter.get_stnear_withdraw(&entity);
        if available_stnear == 0 {
            let total_supporter_rewards = self.internal_get_supporter_rewards(
                &supporter_id,
                &kickstarter,
                goal.tokens_to_release_per_stnear,
            );
            if total_supporter_rewards == amount_to_withdraw {
                let mut supporter = self.internal_get_supporter(&supporter_id);
                supporter.supported_projects.remove(&kickstarter.id);
                self.supporters.insert(&supporter_id, &supporter);
            }
        } 
    }

    #[private]
    pub fn return_tokens_from_kickstarter_callback(
        &mut self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        let amount = amount.0;
        let supporter_id = supporter_id.to_string();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let supporter = self.internal_get_supporter(&supporter_id);
                if supporter.is_empty() {
                    self.supporters.remove(&supporter_id);
                    log!("GODSPEED: {} is no longer part of Katherine!", &supporter_id);
                }
                log!(
                    "CLAIM: {} pTOKEN transfered to Supporter {}",
                    amount, supporter_id
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} pToken not transfered. Recovering {} state.",
                    amount, supporter_id
                );
                self.internal_restore_supporter_withdraw_from_kickstarter(
                    amount,
                    kickstarter_id,
                    supporter_id,
                )
            }
        }
    }

    fn internal_restore_supporter_withdraw_from_kickstarter(
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

        // If the claim fails, add the Kickstarter back to the supported projects.
        let mut supporter = self.internal_get_supporter(&supporter_id);
        if !supporter.supported_projects.contains(&kickstarter.id) {
            supporter.supported_projects.insert(&kickstarter.id);
            self.supporters.insert(&supporter_id, &supporter);
        }
    }
}

/****************************************/
/*  Kickstarter Claim Excedent pTokens  */
/****************************************/

#[near_bindgen]
impl KatherineFundraising {
    pub(crate) fn internal_withdraw_excedent(
        &mut self,
        kickstarter: &mut Kickstarter,
        excedent: Balance
    ) {
        kickstarter.available_reward_tokens -= excedent;
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);

        let excedent = BalanceJSON::from(excedent);
        nep141_token::ft_transfer(
            convert_to_valid_account_id(env::predecessor_account_id()),
            excedent,
            Some("withdraw excedent from kickstarter".to_string()),
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        ).then(
            ext_self_kickstarter::kickstarter_withdraw_excedent_callback(
                kickstarter.id,
                excedent,
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
                log!(
                    "CLAIM: {} pTOKEN transfered to Kickstarter {}",
                    amount, kickstarter_id
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} pToken not transfered. Recovering Kickstarter {} state.",
                    amount, kickstarter_id
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
}

/*********************************/
/*  Katherine Claim pTokens Fee  */
/*********************************/

#[near_bindgen]
impl KatherineFundraising {

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
                    "WITHDRAW: {} pToken withdraw from KickstarterId {} to Account {}",
                    amount,
                    kickstarter_id,
                    env::predecessor_account_id(),
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} pToken not transfered. Recovering Kickstarter {} state.",
                    amount, kickstarter_id
                );
                let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
                kickstarter.katherine_fee = Some(amount);
                self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
            }
        }
    }
}