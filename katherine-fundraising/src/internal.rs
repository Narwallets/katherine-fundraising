use near_sdk::{log, AccountId};
use near_sdk::serde_json::{json};

use crate::*;
use crate::{types::*, errors::*};

impl KatherineFundraising {
    pub fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }
}

/***************************************/
/* Internal methods staking-pool trait */
/***************************************/
impl KatherineFundraising {
    pub(crate) fn internal_deposit(&mut self, amount: Balance) {
        self.assert_min_deposit_amount(amount);
        self.internal_deposit_stnear_into(env::predecessor_account_id(), amount);
    }

    pub(crate) fn internal_deposit_stnear_into(&mut self, supporter_id: SupporterId, amount: Balance) {
        let mut supporter = self.internal_get_supporter(&supporter_id);

        supporter.available += amount;
        self.total_available += amount;

        self.internal_update_supporter(&supporter_id, &supporter);

        log!(
            "{} deposited into @{}'s account. New available balance is {}",
            amount,
            supporter_id,
            supporter.available
        );
    }

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
        self.kickstarters.get(kickstarter_id).expect("Unknown kickstarter id")
    }

    /// Inner method to save the given kickstarter for a given KickstarterID.
    /// If the supporter balances are 0, the supporter is deleted instead to release storage.
    //pub(crate) fn internal_update_supporter(&mut self, supporter_id: &AccountId, supporter: &Supporter) {
    //    if supporter.is_empty() {
    //        self.supporters.remove(supporter_id);
    //    } else {
    //        self.supporters.insert(supporter_id, &supporter); //insert_or_update
    //    }
    //}

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
        kickstarter.total_deposited += amount;
        kickstarter.update_supporter_deposits(&supporter_id, amount);

        // Return unused amount.
        Ok(0)
    }

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

    // pub(crate) fn internal_evaluate_at_due(&mut self) {
    //     // TODO: While this function is running all deposits/withdraws must be frozen.
    //     let active_projects: Vec<Kickstarter> = self.kickstarters
    //         .to_vec()
    //         .into_iter()
    //         .filter(|kickstarter| kickstarter.active)
    //         .collect();
    //     for kickstarter in active_projects.iter() {
    //         if kickstarter.close_timestamp <= env::block_timestamp() {
    //             let kickstarter_id = kickstarter.id;
    //             let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
    //             if kickstarter.evaluate_goals() {
    //                 assert!(
    //                     kickstarter.available_tokens > kickstarter.get_tokens_to_release(),
    //                     "Not enough available tokens to back the supporters rewards"
    //                 );

    //                 log!("The project {} with id: {} was successful!", kickstarter.name, kickstarter_id);
    //                 kickstarter.set_katherine_fee();
    //                 kickstarter.active = false;
    //                 kickstarter.successful = true;
    //                 self.internal_locking_supporters_funds(&kickstarter)
    //             } else {
    //                 log!("The project {} with id: {} was unsuccessful!", kickstarter.name, kickstarter_id);
    //                 kickstarter.active = false;
    //                 kickstarter.successful = false;
    //                 // Instead of freeing funds, if successful is false, then deposits are available for users.
    //                 // self.internal_freeing_supporters_funds(&kickstarter)
    //             }
    //         }
    //     }
    // }

    // pub(crate) fn internal_locking_supporters_funds(&mut self, kickstarter: &Kickstarter) {
    //     let deposits = kickstarter.get_deposits();
    //     for (supporter_id, total) in deposits.iter() {
    //         // Disperse NEAR denominated IOU Note.
    //         let iou_note_id = self.internal_create_iou_note(
    //             &supporter_id,
    //             &kickstarter.id,
    //             &kickstarter.convert_stnear_to_near(&total),
    //             kickstarter.cliff_timestamp,
    //             kickstarter.vesting_timestamp,
    //         );
    //         let mut supporter = self.internal_get_supporter(&supporter_id);
    //         supporter.total_in_deposits -= total;
    //         supporter.locked += total; // <- Not sure if we should keep track of this value.
    //         supporter.iou_note_ids.push(&iou_note_id);

    //         // Disperse Kickstarter Token denominated IOU Note.
    //         let iou_note_id = self.internal_create_iou_note(
    //             &supporter_id,
    //             &kickstarter.id,
    //             &kickstarter.convert_stnear_to_token_shares(&total),
    //             kickstarter.get_token_denomination().clone(),
    //             kickstarter.get_reward_cliff_timestamp(),
    //             kickstarter.get_reward_end_timestamp(),
    //         );
    //         supporter.iou_note_ids.push(&iou_note_id);
    //     }
    // }

    pub(crate) fn internal_freeing_supporters_funds(&mut self, kickstarter: &Kickstarter) {
        // These are too many operations just to free the funds!
        let deposits = kickstarter.get_deposits();
        for (supporter_id, total) in deposits.to_vec().iter() {
            let mut supporter = self.internal_get_supporter(supporter_id);
            supporter.total_in_deposits -= total;
            supporter.available += total;
        }
    }

    // pub(crate) fn transfer_back_to_account(&mut self, account_id: &AccountId, account: &mut Account) {
    //     let available: Balance = account.available;
    //     Promise::new(account_id.to_string()).transfer(available);
    //     account.available -= available;
    //     self.total_available -= available;
    //     self.internal_update_account(&account_id, &account);
    // }

    // pub(crate) fn internal_stake_funds(&mut self) {
    //     Promise::new(self.metapool_contract_address.clone().to_string()).function_call(
    //         b"deposit_and_stake".to_vec(),
    //         json!({}).to_string().as_bytes().to_vec(),
    //         self.total_available,
    //         GAS,
    //     );
    // }

    pub(crate) fn internal_withdraw(&mut self, requested_amount: Balance, kickstarter_id: KickstarterId, supporter_id: &AccountId) {
        let mut kickstarter = self.kickstarters.get(kickstarter_id).expect("kickstarted not found");
        assert!(kickstarter.successful != Some(true)); //WIP if its successfull the logic is different

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
    pub(crate) fn restore_withdraw(&mut self, amount: Balance, kickstarter_id: KickstarterId, supporter_id: AccountId) {
        let mut kickstarter = self.kickstarters.get(kickstarter_id).expect("kickstarted not found");
        let mut deposit = kickstarter.deposits.get(&supporter_id).unwrap_or_default();

        deposit += amount;
        kickstarter.deposits.insert(&supporter_id, &deposit);
    }
}
