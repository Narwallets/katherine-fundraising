use crate::*;
use near_sdk::{near_bindgen, AccountId};

use crate::interface::*;

/// Withdraw is only for **stNear** in Katherine.

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
    fn supporter_withdraw_before_freeze(
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
        let supporter_id = supporter_id.to_string();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let supporter = self.internal_get_supporter(&supporter_id);
                if supporter.is_empty() {
                    self.supporters.remove(&supporter_id);
                    log!("GODSPEED: {} is no longer part of Katherine!", &supporter_id);
                }
                log!("WITHDRAW: {} stNEAR transfer to {}", amount, &supporter_id);
            },
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} stNEAR of interest not transfered. Recovering {} state.",
                    amount, supporter_id
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
        assert!(
            available_to_withdraw > 0
                && requested_amount <= available_to_withdraw,
            "Not available amount!"
        );
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
    fn supporter_withdraw_after_unfreeze(
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
        
        // Do not remove from supported projects unless no more rewards.
        if available_to_withdraw == requested_amount {
            self.remove_from_supported_withdraw(
                kickstarter,
                &supporter_id
            );
        }

        self.kickstarters.replace(kickstarter.id as u64, &kickstarter);
    }

    fn remove_from_supported_withdraw(
        &mut self,
        kickstarter: &mut Kickstarter,
        supporter_id: &SupporterId
    ) {
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

    #[private]
    pub fn return_tokens_after_unfreeze_callback(
        &mut self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
        amount: BalanceJSON
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
                log!("WITHDRAW: {} stNEAR transfer to {}", amount, &supporter_id);
            },
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} stNEAR of interest not transfered. Recovering {} state.",
                    amount, supporter_id
                );
                self.internal_restore_withdraw_after_unfreeze(
                    amount,
                    kickstarter_id,
                    supporter_id
                );
            },
        };
    }

    fn internal_restore_withdraw_after_unfreeze(
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
