use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, Timestamp, PromiseResult
};

mod constants;
mod errors;
mod types;
pub mod internal;
mod metapool;

pub mod interface;
pub mod supporter;
pub mod kickstarter;
pub mod goal;
pub mod utils;
pub use crate::utils::*;

use crate::{constants::*, goal::*, kickstarter::*, metapool::*, supporter::*, types::*, utils::*, interface::*};
pub use metapool::{ext_self, ext_metapool};


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {
    pub owner_id: AccountId,
    pub supporters: UnorderedMap<SupporterId, Supporter>,
    pub kickstarters: Vector<Kickstarter>,
    pub total_available: Balance,

    /// Min amount accepted for supporters
    pub min_deposit_amount: Balance,
    pub metapool_contract_address: AccountId,

    // Katherine fee is a % of the Kickstarter Token rewards.
    // Percent is denominated in basis points 100% equals 10_000 basis points.
    pub katherine_fee_percent: u32,
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(
        owner_id: AccountId,
        staking_goal: Balance,
        min_deposit_amount: Balance,
        metapool_contract_address: AccountId,
        katherine_fee_percent: u32
    ) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(b"Supporters".to_vec()),
            kickstarters: Vector::new(b"Kickstarters".to_vec()),
            total_available: 0,
            min_deposit_amount,
            metapool_contract_address,
            katherine_fee_percent,
        }
    }

    pub fn withdraw_kickstarter_tokens(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let account = env::predecessor_account_id();
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        self.internal_withdraw_kickstarter_tokens(amount.into(), &mut kickstarter, &account);

        nep141_token::ft_transfer_call(
            account.clone(),
            amount,
            Some("withdraw from kickstarter".to_string()),
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        // restore user balance on error
        .then(ext_self_kikstarter::return_tokens_from_kickstarter_callback(
            account.clone(),
            kickstarter_id,
            amount,
            &env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER
        ));
    }

    #[private]
    pub fn return_tokens_from_kickstarter_callback(&mut self, user: AccountId, kickstarter_id: KickstarterIdJSON, amount: U128){
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount.0
                );
                self.internal_restore_kickstarter_withdraw(amount.into(), kickstarter_id.into(), user)
            }
        }
    }

    /***************************/
    /*    Withdraw methods     */
    /***************************/

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let account = env::predecessor_account_id();
        self.internal_withdraw(amount.into(), kickstarter_id.into(), &account);
        nep141_token::ft_transfer_call(
            account.clone(),
            amount,
            Some("withdraw from kickstarter".to_string()),
            &self.metapool_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        // restore user balance on error
        .then(ext_self_metapool::return_tokens_callback(
            account.clone(),
            kickstarter_id,
            amount,
            &env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER
        ));
    }

    #[private]
    pub fn return_tokens_callback(&mut self, user: AccountId, kickstarter_id: KickstarterIdJSON, amount: BalanceJSON) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount.0
                );
                self.internal_restore_withdraw(amount.into(), kickstarter_id.into(), user)
            }
        }
    }

    /************************/
    /*    Robot methods     */
    /************************/

    /// Returns both successfull and unsuccessfull kickstarter ids in a single struc 
    pub fn get_kickstarters_to_process(
        &self,
        from_index: KickstarterIdJSON,
        limit: KickstarterIdJSON
    ) -> Option<KickstarterStatusJSON> {
        let kickstarters_len = self.kickstarters.len();
        let start: u64 = from_index.into();
        if start >= kickstarters_len {
            return None;
        }
        let mut successful: Vec<KickstarterIdJSON> = Vec::new();
        let mut unsuccessful: Vec<KickstarterIdJSON> = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, kickstarters_len) {
            let kickstarter = self.kickstarters
                .get(index)
                .expect("internal error, kickstarter ID is out of range");
            if kickstarter.active && kickstarter.close_timestamp <= get_epoch_millis() {
                if kickstarter.any_achieved_goal() {
                    successful.push(KickstarterIdJSON::from(kickstarter.id))
                }
                else{
                    unsuccessful.push(KickstarterIdJSON::from(kickstarter.id))
                }
            }
        }
        Some(KickstarterStatusJSON{successful, unsuccessful})
    }

    pub fn process_kickstarter(&mut self, kickstarter_id: KickstarterIdJSON) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful.is_none() {
            if kickstarter.any_achieved_goal() {
                self.internal_activate_kickstarter(kickstarter_id.into());
                log!("kickstarter was successfully activated");
            } else {
                kickstarter.active = false;
                kickstarter.successful = Some(false);
                log!("kickstarter successfully deactivated");
            }
        } else {
            panic!("kickstarter already activated");
        }
    }

    /*****************************/
    /*   Supporters functions   */
    /*****************************/

    pub fn get_supporter_total_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON
    ) -> Balance {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.kickstarters.to_vec().contains(&kickstarter.id) {
                    let goal = kickstarter.get_goal();
                    return self.internal_get_supporter_total_rewards(
                        &supporter_id,
                        &kickstarter,
                        goal,
                    );
                } else {
                    panic!("Supporter is not part of Kickstarter!");
                }
            },
            None => panic!("Supporter does not have any reward!"),
        }
    }

    pub fn get_supporter_available_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON
    ) -> Balance {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.kickstarters.to_vec().contains(&kickstarter.id) {
                    let goal = kickstarter.get_goal();
                    let total_rewards = self.internal_get_supporter_total_rewards(
                        &supporter_id,
                        &kickstarter,
                        goal,
                    );
                    let supporter_withdraw: Balance = match kickstarter.withdraw.get(&supporter_id) {
                        Some(value) => value,
                        None => 0,
                    };
                    return total_rewards - supporter_withdraw;
                } else {
                    panic!("Supporter is not part of Kickstarter!");
                }
            },
            None => panic!("Supporter does not have any reward!"),
        }
    }

    /*****************************/
    /*   Kickstarter functions   */
    /*****************************/

    pub fn kickstarter_withdraw_excedent(&self, kickstarter_id: KickstarterIdJSON){
        let kickstarter = self.kickstarters.get(kickstarter_id.into()).expect("kickstarter not found");
        only_kickstarter_admin(&kickstarter);

        
        let excedent = kickstarter.available_reward_tokens - kickstarter.get_goal().tokens_to_release;

        if excedent > 0 {
            nep141_token::ft_transfer_call(
                env::predecessor_account_id().clone(),
                excedent.into(),
                Some("withdraw excedent from kickstarter".to_string()),
                &kickstarter.token_contract_address,
                1,
                GAS_FOR_FT_TRANSFER,
            )
            // restore user balance on error
            .then(ext_self_kikstarter::kickstarter_withdraw_excedent_callback(
                kickstarter_id,
                excedent.into(),
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER
            ));
        }

        
    }

    pub fn kickstarter_withdraw_excedent_callback(&mut self, kickstarter_id: KickstarterIdJSON, amount: U128){
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            },
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount.0
                );
                self.internal_restore_kickstarter_excedent_withdraw(amount.into(), kickstarter_id.into())
            }
        }
    }

    pub fn get_kickstarters(&self, from_index: usize, limit: usize) -> Vec<KickstarterJSON> {
        let kickstarters_len = self.kickstarters.len() as usize;
        assert!(from_index <= kickstarters_len, "from_index is out of range!");
        let mut results: Vec<KickstarterJSON> = Vec::new();
        for index in from_index..std::cmp::min(from_index + limit, kickstarters_len) {
            let kickstarter = self.kickstarters
                .get(index as u64)
                .expect("Kickstarter ID is out of range!");
                results.push(
                    KickstarterJSON {
                        id: kickstarter.id.into(),
                        total_supporters: U64String::from(kickstarter.total_supporters)
                    }
                );
        }
        results
    }

    pub fn get_kickstarter(&self, kickstarter_id: KickstarterIdJSON) -> KickstarterJSON {
        let kickstarters_len = self.kickstarters.len();
        assert!(kickstarter_id as u64 <= kickstarters_len, "from_index is out of range!");
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        KickstarterJSON {
            id: kickstarter.id.into(),
            total_supporters: U64String::from(kickstarter.total_supporters)
        }
    }

    /// Creates a new kickstarter entry in persistent storage
    pub fn create_kickstarter(
        &mut self,
        name: String,
        slug: String,
        owner_id: AccountId,
        finish_timestamp: Timestamp,
        open_timestamp: Timestamp,
        close_timestamp: Timestamp,
        vesting_timestamp: Timestamp,
        cliff_timestamp: Timestamp,
        token_contract_address: AccountId,
    ) {
        self.only_admin();
        let kickstarter = Kickstarter {
            id: self.kickstarters.len() as u32,
            name,
            slug,
            goals: Vector::new(b"Goal".to_vec()),
            winner_goal_id: None,
            katherine_fee: None,
            supporters: Vec::new(),
            total_supporters: 0,
            deposits: UnorderedMap::new(b"A".to_vec()),
            withdraw: UnorderedMap::new(b"W".to_vec()),
            total_deposited: 0,
            owner_id,
            active: true,
            successful: None,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
        };

        self.kickstarters.push(&kickstarter);
    }

    #[allow(unused)]
    pub fn delete_kickstarter(&mut self, id: KickstarterId) {
        panic!("Kickstarter must not be deleted!");
    }

    /// Update a kickstarter
    pub fn update_kickstarter(
        &mut self,
        id: KickstarterId,
        name: String,
        slug: String,
        owner_id: AccountId,
        finish_timestamp: Timestamp,
        open_timestamp: Timestamp,
        close_timestamp: Timestamp,
        vesting_timestamp: Timestamp,
        cliff_timestamp: Timestamp,
        token_contract_address: AccountId,
    ) {
        self.only_admin();
        let old_kickstarter = self.internal_get_kickstarter(id);
        assert!(
            old_kickstarter.open_timestamp <= env::block_timestamp(),
            "Changes are not allow after the funding period started!"
        );
        assert!(
            self.owner_id != env::predecessor_account_id(),
            "Only Katherine owner is allowed to modify the project!"
        );

        let kickstarter = Kickstarter {
            id,
            name,
            slug,
            goals: Vector::new(b"Goal".to_vec()),
            winner_goal_id: None,
            katherine_fee: None,
            supporters: Vec::new(),
            total_supporters: 0,
            deposits: UnorderedMap::new(b"A".to_vec()),
            withdraw: UnorderedMap::new(b"W".to_vec()),
            total_deposited: 0,
            owner_id,
            active: true,
            successful: None,
            stnear_value_in_near: None,
            creation_timestamp: env::block_timestamp(),
            finish_timestamp,
            open_timestamp,
            close_timestamp,
            vesting_timestamp,
            cliff_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
        };

        self.kickstarters.replace(id as u64, &kickstarter);
    }

    /********************/
    /*   View methods   */
    /********************/

    pub fn get_total_kickstarters(&self) -> u32 {
        return self.kickstarters.len() as u32;
    }
}


// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {

//     use near_sdk::{testing_env, MockedBlockchain, VMContext};

//     mod unit_test_utils;
//     use unit_test_utils::*;

//     use super::*;


//     /// Get initial context for tests
//     fn basic_context() -> VMContext {
//         println!("SYSTEM ACCOUNT: {}", SYSTEM_ACCOUNT.to_string());
//         get_context(
//             SYSTEM_ACCOUNT.into(),
//             ntoy(TEST_INITIAL_BALANCE),
//             0,
//             to_ts(GENESIS_TIME_IN_DAYS),
//             false,
//         )
//     }

//     /// Creates a new contract
//     fn new_contract() -> KatherineFundraising {
//         KatherineFundraising::new(
//             OWNER_ACCOUNT.into(),
//         )
//     }

//     fn contract_only_setup() -> (VMContext, KatherineFundraising) {
//         let context = basic_context();
//         testing_env!(context.clone());
//         let contract = new_contract();
//         return (context, contract);
//     }


//     #[test]
//     fn test_create_kickstarter() {
//         let (_context, mut contract) = contract_only_setup();
//         _new_kickstarter(_context, &mut contract);
//         assert_eq!(1, contract.kickstarters.len());
//     }

//     #[test]
//     fn test_create_supporter() {
//         let (_context, mut contract) = contract_only_setup();
//         _new_kickstarter(_context, &mut contract);
//         let kickstarter_id = contract.kickstarters.len() - 1;
//         contract.kickstarters.get(kickstarter_id).unwrap()
//             .update_supporter_deposits(&String::from(SUPPORTER_ACCOUNT), &DEPOSIT_AMOUNT)
//     }

//     #[test]
//     fn test_workflow() {
//         let step: u64 = 50;
//         // TODO: create a function for this setup
//         let (_context, mut contract) = contract_only_setup();
//         _new_kickstarter(_context, &mut contract);
//         let kickstarter_id = contract.kickstarters.len() - 1;
//         //TODO
//         setup_succesful_kickstarter_configuration(&mut contract);

//         let total_ks: u64 = u64::from(contract.get_total_kickstarters());
//         let mut start: u64 = 0;
//         let mut end: u64 = u64::min(step, total_ks);
//         while end <= total_ks {
//             let ready_ks = contract.get_kickstarter_ids_ready_to_eval(start, end);
//             let (successful_ks, unsuccessful_ks) = contract.get_evaluated_kickstarters_from(ready_ks);
//             test_activate_kickstarters(&successful_ks, &mut contract);
//             test_deactivate_kickstarters(&unsuccessful_ks, &mut contract);
//             test_disperse_iou_notes(&successful_ks, &mut contract);
//             start = end;
//             end = std::cmp::min(start + step, u64::from(total_ks));
//         }
//     }

//     fn test_disperse_iou_notes(kickstarters: &Vec<KickstarterJSON>, contract: &mut KatherineFundraising) {
//         let step: u64 = 50;
//         use std::convert::TryFrom;
//         for k in kickstarters.iter() {
//             let mut start: u64 = 0;
//             let mut end: u64 = std::cmp::min(step, u64::from(k.total_supporters));
            
//             while end <= u64::from(k.total_supporters) {
//                 let supporters = contract.get_kickstarter_supporters(
//                     k.id,
//                     start,
//                     end,
//                 );
//                 contract.disperse_iou_notes_to_supporters(supporters);
//                 let mut start = end;
//                 end = std::cmp::min(start + step, u64::from(k.total_supporters));
//             }
//         }    
//     }    

//     fn setup_succesful_kickstarter_configuration(contract: &mut KatherineFundraising) {
//         println!("TODO: implement successful kickstarter configuration");
//     }

//     fn test_activate_kickstarters(kickstarters: &Vec<KickstarterJSON>, contract: &mut KatherineFundraising) {
//         for k in kickstarters {
//             let active_ks = contract.activate_successful_kickstarter(k.id);
//             assert_eq!(true, active_ks);
//         }
//     }

//     fn test_deactivate_kickstarters(kickstarters: &Vec<KickstarterJSON>, contract: &mut KatherineFundraising) {
//         for k in kickstarters {
//             let unactive_ks = contract.deactivate_unsuccessful_kickstarter(k.id);
//             assert_eq!(true, unactive_ks);
//         }
//     }
// }
