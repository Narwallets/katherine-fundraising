use crate::*;
use near_sdk::{log, AccountId};
use near_sdk::Promise;
use near_sdk::serde_json::{json};

use crate::iou_note::IOUNoteDenomination;

pub use crate::types::*;

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

    pub(crate) fn internal_deposit_stnear_into(&mut self, supporter_id: AccountId, amount: Balance) {
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
    pub(crate) fn internal_get_supporter(&self, supporter_id: &AccountId) -> Supporter {
        self.supporters.get(supporter_id).unwrap_or_default()
    }

    /// Inner method to save the given supporter for a given supporter ID.
    /// If the supporter balances are 0, the supporter is deleted instead to release storage.
    pub(crate) fn internal_update_supporter(&mut self, supporter_id: &AccountId, supporter: &Supporter) {
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
        kickstarter_id: String
    ) -> Result<Balance, String> {
        let kickstarter_id = match kickstarter_id.parse::<KickstarterId>() {
            Ok(_id) => _id,
            Err(_) => return Err("Invalid Kickstarter id.".into()),
        };

        let mut kickstarter: Kickstarter = match self.kickstarters.get(kickstarter_id) {
            Some(kickstarter) => kickstarter,
            None => return Err("Kickstarter id not found.".into()),
        };

        let current_timestamp = env::block_timestamp();
        if current_timestamp >= kickstarter.close_timestamp || current_timestamp < kickstarter.open_timestamp {
            return Err("Not within the funding period.".into());
        }

        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        kickstarter.update_supporter_deposits(&supporter_id, amount);

        let unused_amount: Balance = 0;
        Ok(unused_amount)
    }

    pub(crate) fn internal_evaluate_at_due(&mut self) {
        let active_projects: Vec<Kickstarter> = self.kickstarters.to_vec().into_iter().filter(|kickstarter| kickstarter.active).collect();
        for kickstarter in active_projects.iter() {
            if kickstarter.close_timestamp <= env::block_timestamp() {
                let kickstarter_id = kickstarter.id;
                let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
                if kickstarter.evaluate_goals() {
                    log!("The project {} with id: {} was successful!", kickstarter.name, kickstarter_id);
                    kickstarter.active = false;
                    kickstarter.succesful = true;
                    self.internal_locking_supporters_funds(&kickstarter)
                } else {
                    log!("The project {} with id: {} was unsuccessful!", kickstarter.name, kickstarter_id);
                    kickstarter.active = false;
                    kickstarter.succesful = false;
                    self.internal_freeing_supporters_funds(&kickstarter)
                }
            }
        }
    }

    pub(crate) fn internal_locking_supporters_funds(&mut self, kickstarter: &Kickstarter) {
        let deposits = kickstarter.get_deposits();
        for (supporter_id, total) in deposits.to_vec().iter() {
            // Disperse NEAR denominated IOU Note.
            let iou_note_id = self.internal_create_iou_note(
                supporter_id,
                &kickstarter.id,
                &kickstarter.convert_stnear_to_near(total),
                IOUNoteDenomination::NEAR,
                kickstarter.cliff_timestamp,
                kickstarter.vesting_timestamp,
            );
            let mut supporter = self.internal_get_supporter(supporter_id);
            supporter.total_in_deposits -= total;
            supporter.locked += total; // <- Not sure if we should keep track of this value.
            supporter.iou_note_ids.push(&iou_note_id);

            // Disperse Kickstarter Token denominated IOU Note.
            let iou_note_id = self.internal_create_iou_note(
                supporter_id,
                &kickstarter.id,
                &kickstarter.convert_stnear_to_token_shares(total),
                IOUNoteDenomination::NEAR,
                kickstarter.cliff_timestamp,
                kickstarter.vesting_timestamp,
            );
        }
    }

    pub(crate) fn internal_freeing_supporters_funds(&mut self, kickstarter: &Kickstarter) {
        let deposits = kickstarter.get_deposits();
        for (supporter_id, total) in deposits.to_vec().iter() {
            let mut supporter = self.internal_get_supporter(supporter_id);
            supporter.total_in_deposits -= total;
            supporter.available += total;
        }
    }

    pub(crate) fn internal_create_iou_note(
        &mut self,
        supporter_id: &SupporterId,
        kickstarter_id: &KickstarterId,
        amount: &Balance,
        denomination: IOUNoteDenomination,
        cliff_timestamp: Timestamp,
        end_timestamp: Timestamp,
    ) -> IOUNoteId {
        let iou_note = IOUNote {
            id: self.iou_notes.len(),
            amount: amount.clone(),
            denomination,
            supporter_id: supporter_id.clone(),
            kickstarter_id: kickstarter_id.clone(),
            cliff_timestamp,
            end_timestamp,
        };
        self.iou_notes.push(&iou_note);
        iou_note.id
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

    pub(crate) fn internal_withdraw(&mut self, requested_amount: Balance) -> Promise {
        let supporter_id = env::predecessor_account_id();
        let mut supporter = self.internal_get_supporter(&supporter_id);

        let amount = supporter.take_from_available(requested_amount, self);
        self.internal_update_supporter(&supporter_id, &supporter);
        Promise::new(supporter_id)
    }
}
