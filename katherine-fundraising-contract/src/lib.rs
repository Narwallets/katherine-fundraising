use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult};

mod constants;
mod internal;
mod metapool;
mod types;

pub mod goal;
pub mod interface;
pub mod kickstarter;
pub mod supporter;
pub mod utils;
pub use crate::utils::*;

use crate::{constants::*, goal::*, interface::*, kickstarter::*, supporter::*, types::*};


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {
    pub owner_id: AccountId,
    pub supporters: UnorderedMap<SupporterId, Supporter>,
    pub kickstarters: Vector<Kickstarter>,
    pub kickstarter_id_by_slug: UnorderedMap<String, KickstarterId>,
    pub total_available: Balance,

    /// Min amount accepted for supporters
    pub min_deposit_amount: Balance,
    pub metapool_contract_address: AccountId,

    // Katherine fee is a % of the Kickstarter Token rewards.
    // Percent is denominated in basis points 100% equals 10_000 basis points.
    pub katherine_fee_percent: BasisPoints,
    pub max_goals_per_kickstarter: u8,

    // Active kickstarter projects.
    pub active_projects: UnorderedSet<KickstarterId>,
}

#[near_bindgen]
impl KatherineFundraising {
    #[init]
    pub fn new(
        owner_id: AccountId,
        min_deposit_amount: BalanceJSON,
        metapool_contract_address: AccountId,
        katherine_fee_percent: BasisPoints,
    ) -> Self {
        // assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(Keys::Supporters),
            kickstarters: Vector::new(Keys::Kickstarters),
            kickstarter_id_by_slug: UnorderedMap::new(Keys::KickstarterId),
            total_available: 0,
            min_deposit_amount: Balance::from(min_deposit_amount),
            metapool_contract_address,
            katherine_fee_percent,
            max_goals_per_kickstarter: 5,
            active_projects: UnorderedSet::new(Keys::Active),
        }
    }

    /************************/
    /*    Robot methods     */
    /************************/

    /// Returns both successful and unsuccessful kickstarter ids in a single struc.
    pub fn get_kickstarters_to_process(
        &self,
        from_index: KickstarterIdJSON,
        limit: KickstarterIdJSON,
    ) -> Option<KickstarterStatusJSON> {
        let kickstarters_len = self.kickstarters.len();
        let start: u64 = from_index.into();
        if start >= kickstarters_len {
            return None;
        }
        let mut successful: Vec<KickstarterIdJSON> = Vec::new();
        let mut unsuccessful: Vec<KickstarterIdJSON> = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, kickstarters_len) {
            let kickstarter = self.internal_get_kickstarter(index as u32);
            if kickstarter.active && kickstarter.close_timestamp <= get_current_epoch_millis() {
                if kickstarter.any_achieved_goal() {
                    successful.push(KickstarterIdJSON::from(kickstarter.id));
                } else {
                    unsuccessful.push(KickstarterIdJSON::from(kickstarter.id));
                }
            }
        }
        Some(KickstarterStatusJSON {
            successful,
            unsuccessful,
        })
    }

    pub fn process_kickstarter(&mut self, kickstarter_id: KickstarterIdJSON) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful.is_none() {
            if kickstarter.close_timestamp <= get_current_epoch_millis() {
                match kickstarter.get_achieved_goal() {
                    Some(goal) => {
                        self.activate_successful_kickstarter(kickstarter_id, goal.id);
                        log!("kickstarter was successfully activated");
                    },
                    None => {
                        kickstarter.active = false;
                        self.active_projects.remove(&kickstarter.id);
                        kickstarter.successful = Some(false);
                        self.kickstarters
                            .replace(kickstarter_id as u64, &kickstarter);
                        log!("kickstarter successfully deactivated");                    
                    },
                }
            } else {
                panic!("Funding period is not over!")
            }
        } else {
            panic!("kickstarter already activated");
        }
    }

    /// Start the cross-contract call to unfreeze the kickstarter funds.
    pub fn unfreeze_kickstarter_funds(&mut self, kickstarter_id: KickstarterIdJSON) {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful == Some(true) && kickstarter.stnear_price_at_unfreeze == None {
            kickstarter.assert_funds_can_be_unfreezed();
            self.internal_unfreeze_kickstarter_funds(kickstarter_id);
        }
    }

    /*****************************/
    /*   Supporters functions    */
    /*****************************/

    pub fn withdraw_all(&mut self, kickstarter_id: KickstarterIdJSON) {
        let supporter_id = convert_to_valid_account_id(env::predecessor_account_id());
        let amount = self.get_supporter_total_deposit_in_kickstarter(supporter_id, kickstarter_id);
        self.withdraw(amount, kickstarter_id);
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let min_prepaid_gas = GAS_FOR_FT_TRANSFER + GAS_FOR_RESOLVE_TRANSFER + FIVE_TGAS;
        assert!(env::prepaid_gas() > min_prepaid_gas, "gas required {}", min_prepaid_gas);
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        let amount = Balance::from(amount);
        let supporter_id: SupporterId = env::predecessor_account_id();
        let deposit = kickstarter.get_deposit(&supporter_id);
        let (amount_to_remove, amount_to_send) = match kickstarter.successful {
            Some(true) => {
                kickstarter.assert_funds_must_be_unfreezed();
                let price_at_freeze = kickstarter.stnear_price_at_freeze.expect("Price at freeze is not defined!");
                let price_at_unfreeze = kickstarter.stnear_price_at_unfreeze.expect("Price at unfreeze is not defined!");
                let max_amount_to_withdraw = proportional(deposit, price_at_freeze, price_at_unfreeze);
                assert!(amount <= max_amount_to_withdraw, "Not available amount!");
                if is_close(amount, max_amount_to_withdraw) {
                    (deposit, max_amount_to_withdraw)
                } else {
                    (
                        proportional(amount, price_at_unfreeze, price_at_freeze),
                        amount
                    )
                }
            },
            Some(false) => {
                assert!(amount <= deposit, "Not available amount!");
                if is_close(amount, deposit) {
                    (deposit, deposit)
                } else {
                    (amount, amount)
                }                
            },
            None => {
                assert!(
                    get_current_epoch_millis() < kickstarter.close_timestamp,
                    "The funding period is over, Kickstarter must be evaluated!"
                );
                assert!(amount <= deposit, "Not available amount!");
                if is_close(amount, deposit) {
                    (deposit, deposit)
                } else {
                    (amount, amount)
                }
            }
        };

        self.internal_supporter_withdraw(amount_to_remove, deposit, &mut kickstarter, &supporter_id);
        let supporter_id = convert_to_valid_account_id(supporter_id);
        nep141_token::ft_transfer(
            supporter_id.clone(),
            BalanceJSON::from(amount_to_send),
            None,
            &self.metapool_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        // restore user balance on error
        .then(ext_self_metapool::return_tokens_callback(
            supporter_id,
            kickstarter_id,
            BalanceJSON::from(amount_to_remove),
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_TRANSFER
        ));
    }

    #[private]
    pub fn return_tokens_callback(&mut self, user: SupporterIdJSON, kickstarter_id: KickstarterIdJSON, amount: BalanceJSON) {
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
                let supporter_id: SupporterId = user.to_string();
                self.internal_restore_withdraw(amount.into(), kickstarter_id.into(), supporter_id)
            }
        }
    }

    // lets supporters withdraw the tokens emited by the kickstarter
    pub fn withdraw_kickstarter_tokens(
        &mut self,
        amount: BalanceJSON,
        kickstarter_id: KickstarterIdJSON,
    ) {
        let account = env::predecessor_account_id();
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        self.internal_withdraw_kickstarter_tokens(amount.into(), &mut kickstarter, &account);

        nep141_token::ft_transfer_call(
            convert_to_valid_account_id(account.clone()),
            amount,
            None,
            "withdraw from kickstarter".to_string(),
            &kickstarter.token_contract_address,
            1,
            GAS_FOR_FT_TRANSFER,

        )
        // restore user balance on error
        .then(
            ext_self_kickstarter::return_tokens_from_kickstarter_callback(
                convert_to_valid_account_id(account.clone()),
                kickstarter_id,
                amount,
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER,
            ),
        );
    }

    #[private]
    pub fn return_tokens_from_kickstarter_callback(
        &mut self,
        user: AccountId,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering account state",
                    amount.0
                );
                self.internal_restore_supporter_withdraw_from_kickstarter(
                    amount.into(),
                    kickstarter_id.into(),
                    user,
                )
            }
        }
    }

    /*****************************/
    /*   Kickstarter functions   */
    /*****************************/

    pub fn withdraw_stnear_interest(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: BalanceJSON,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        kickstarter.assert_kickstarter_owner();
        assert_eq!(kickstarter.successful, Some(true), "Kickstarter is unsuccessful!");

        let amount: Balance = amount.into();

        if let Some(st_near_price) = kickstarter.stnear_price_at_unfreeze {
            // No need to get stnear price from metapool.
            self.internal_kickstarter_withdraw(&mut kickstarter, st_near_price, amount);
        } else {
            assert!(!kickstarter.funds_can_be_unfreezed(), "Unfreeze funds before interest withdraw!");
            // Get stNear price from metapool.
            ext_self_metapool::get_st_near_price(
                &self.metapool_contract_address,
                0,
                GAS_FOR_GET_STNEAR
            )
            .then(ext_self_kickstarter::kickstarter_withdraw_callback(
                kickstarter_id, 
                amount.into(),
                &env::current_account_id(),
                0,
                env::prepaid_gas() - env::used_gas() - GAS_FOR_GET_STNEAR
            ));
        }        
    }

    #[private]
    pub fn kickstarter_withdraw_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
        #[callback] st_near_price: U128
    ){
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        self.internal_kickstarter_withdraw(&mut kickstarter, st_near_price.into(), amount.into());
    }

    #[private]
    pub fn kickstarter_withdraw_resolve_transfer(
        &mut self,
        kickstarter_id: KickstarterIdJSON, 
        amount: U128
    ){
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount.0
                );
                self.internal_restore_kickstarter_withdraw(
                    amount.into(),
                    kickstarter_id.into(),
                )
            }
        }
    }

    pub fn kickstarter_withdraw_excedent(&mut self, kickstarter_id: KickstarterIdJSON) {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        kickstarter.assert_kickstarter_owner();
        assert!(
            kickstarter.close_timestamp < get_current_epoch_millis(),
            "The excedent is avalable only after the funding period ends"
        );

        let excedent: Balance = match kickstarter.successful {
            Some(true) => {
                let katherine_fee = kickstarter.katherine_fee.unwrap();
                let total_tokens_to_release = kickstarter.total_tokens_to_release.unwrap();
                kickstarter.available_reward_tokens - (
                    katherine_fee + total_tokens_to_release
                )
            },
            Some(false) => {
                log!("Returning all available reward tokens!");
                kickstarter.available_reward_tokens
            },
            None => panic!("Before withdrawing pTOKEN, evaluate the project using the process_kickstarter fn!"),
        };

        if excedent > 0 {
            nep141_token::ft_transfer(
                convert_to_valid_account_id(env::predecessor_account_id()),
                excedent.into(),
                Some("withdraw excedent from kickstarter".to_string()),
                &kickstarter.token_contract_address,
                1,
                GAS_FOR_FT_TRANSFER,
            )
            // restore user balance on error
            .then(ext_self_kickstarter::kickstarter_withdraw_excedent_callback(
                kickstarter_id,
                excedent.into(),
                &env::current_account_id(),
                0,
                GAS_FOR_FT_TRANSFER,
            ));
        } else {
            panic!("No remaining excedent pTOKEN!");
        }
    }

    #[private]
    pub fn kickstarter_withdraw_excedent_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("token transfer {}", u128::from(amount));
            }
            PromiseResult::Failed => {
                log!(
                    "token transfer failed {}. recovering kickstarter state",
                    amount.0
                );
                self.internal_restore_kickstarter_excedent_withdraw(
                    amount.into(),
                    kickstarter_id.into(),
                )
            }
        }
    }

    /***********************/
    /*   Admin functions   */
    /***********************/

    /// Creates a new kickstarter entry in persistent storage.
    pub fn create_kickstarter(
        &mut self,
        name: String,
        slug: String,
        owner_id: AccountId,
        open_timestamp: EpochMillis,
        close_timestamp: EpochMillis,
        token_contract_address: AccountId,
        deposits_hard_cap: BalanceJSON,
        max_tokens_to_release_per_stnear: BalanceJSON,
    ) -> KickstarterIdJSON {
        //ONLY ADMINS CAN CREATE KICKSTARTERS? YES
        self.assert_only_admin();
        self.assert_unique_slug(&slug);
        let id = self.kickstarters.len() as KickstarterId;
        let kickstarter = Kickstarter {
            id: id,
            name,
            slug,
            goals: Vector::new(Keys::Goals.as_prefix(&id).as_bytes()),
            winner_goal_id: None,
            katherine_fee: None,
            total_tokens_to_release: None,
            deposits: UnorderedMap::new(Keys::Deposits.as_prefix(&id).as_bytes()),
            withdraw: UnorderedMap::new(Keys::Withdraws.as_prefix(&id).as_bytes()),
            total_deposited: 0,
            deposits_hard_cap: Balance::from(deposits_hard_cap),
            max_tokens_to_release_per_stnear: Balance::from(max_tokens_to_release_per_stnear),
            enough_reward_tokens: false,
            owner_id,
            active: true,
            successful: None,
            stnear_price_at_freeze: None,
            stnear_price_at_unfreeze: None,
            creation_timestamp: get_current_epoch_millis(),
            open_timestamp,
            close_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
            kickstarter_withdraw: 0
        };
        kickstarter.assert_timestamps();
        self.kickstarters.push(&kickstarter);
        self.kickstarter_id_by_slug
            .insert(&kickstarter.slug, &kickstarter.id);
        self.active_projects.insert(&kickstarter.id);
        kickstarter.id.into()
    }

    #[allow(unused)]
    pub fn delete_kickstarter(&mut self, id: KickstarterId) {
        panic!("Kickstarter must not be deleted!");
    }

    pub fn update_kickstarter(
        &mut self,
        id: KickstarterId,
        name: String,
        slug: String,
        owner_id: AccountId,
        open_timestamp: EpochMillis,
        close_timestamp: EpochMillis,
        token_contract_address: AccountId,
        deposits_hard_cap: BalanceJSON,
        max_tokens_to_release_per_stnear: BalanceJSON,
    ) {
        self.assert_only_admin();
        self.assert_unique_slug(&slug);
        let old_kickstarter = self.internal_get_kickstarter(id);
        assert!(
            old_kickstarter.open_timestamp >= get_current_epoch_millis(),
            "Changes are not allow after the funding period started!"
        );

        let kickstarter = Kickstarter {
            id,
            name,
            slug,
            goals: Vector::new(Keys::Goals.as_prefix(&id).as_bytes()),
            winner_goal_id: None,
            katherine_fee: None,
            total_tokens_to_release: None,
            deposits: UnorderedMap::new(Keys::Deposits.as_prefix(&id).as_bytes()),
            withdraw: UnorderedMap::new(Keys::Withdraws.as_prefix(&id).as_bytes()),
            total_deposited: 0,
            deposits_hard_cap: Balance::from(deposits_hard_cap),
            max_tokens_to_release_per_stnear: Balance::from(max_tokens_to_release_per_stnear),
            enough_reward_tokens: false,
            owner_id,
            active: true,
            successful: None,
            stnear_price_at_freeze: None,
            stnear_price_at_unfreeze: None,
            creation_timestamp: get_current_epoch_millis(),
            open_timestamp,
            close_timestamp,
            token_contract_address,
            available_reward_tokens: 0,
            locked_reward_tokens: 0,
            kickstarter_withdraw: 0
        };
        kickstarter.assert_timestamps();
        self.kickstarters.replace(id as u64, &kickstarter);
        self.kickstarter_id_by_slug.remove(&old_kickstarter.slug);
        self.kickstarter_id_by_slug
            .insert(&kickstarter.slug, &kickstarter.id);
    }

    /**********************/
    /*   View functions   */
    /**********************/

    pub fn get_supporter_total_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
    ) -> Balance {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.supported_projects.to_vec().contains(&kickstarter.id) {
                    let goal = kickstarter.get_winner_goal();
                    return self.internal_get_supporter_total_rewards(
                        &supporter_id,
                        &kickstarter,
                        goal,
                    );
                } else {
                    panic!("Supporter is not part of Kickstarter!");
                }
            }
            None => panic!("Supporter does not have any reward!"),
        }
    }

    pub fn get_supporter_available_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
    ) -> Balance {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.supported_projects.to_vec().contains(&kickstarter.id) {
                    let goal = kickstarter.get_winner_goal();
                    let total_rewards = self.internal_get_supporter_total_rewards(
                        &supporter_id,
                        &kickstarter,
                        goal,
                    );
                    let supporter_withdraw: Balance = match kickstarter.withdraw.get(&supporter_id)
                    {
                        Some(value) => value,
                        None => 0,
                    };
                    return total_rewards - supporter_withdraw;
                } else {
                    panic!("Supporter is not part of Kickstarter!");
                }
            }
            None => panic!("Supporter does not have any reward!"),
        }
    }

    pub fn get_active_projects(
        &self,
        from_index: u32,
        limit: u32,
    ) -> Option<ActiveKickstarterJSON> {
        let projects = self.active_projects.to_vec();
        let projects_len = projects.len() as u64;
        let start: u64 = from_index.into();
        if start >= projects_len {
            return None;
        }
        let mut active: Vec<KickstarterJSON> = Vec::new();
        let mut open: Vec<KickstarterJSON> = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, projects_len) {
            let kickstarter_id = projects.get(index as usize).expect("Out of index!");
            let kickstarter = self.internal_get_kickstarter(*kickstarter_id);
            if kickstarter.is_within_funding_period() {
                open.push(kickstarter.to_json());
            } else {
                active.push(kickstarter.to_json());
            }
        }
        Some(ActiveKickstarterJSON { active, open })
    }

    pub fn get_project_details(&self, kickstarter_id: KickstarterIdJSON) -> KickstarterDetailsJSON {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.to_details_json()
    }

    pub fn get_kickstarters(&self, from_index: usize, limit: usize) -> Vec<KickstarterJSON> {
        let kickstarters_len = self.kickstarters.len() as usize;
        assert!(
            from_index <= kickstarters_len,
            "from_index is out of range!"
        );
        let mut results: Vec<KickstarterJSON> = Vec::new();
        for index in from_index..std::cmp::min(from_index + limit, kickstarters_len) {
            let kickstarter = self.internal_get_kickstarter(index as u32);
            results.push(kickstarter.to_json());
        }
        results
    }

    pub fn get_kickstarter(&self, kickstarter_id: KickstarterIdJSON) -> KickstarterJSON {
        let kickstarters_len = self.get_total_kickstarters();
        assert!(kickstarter_id <= kickstarters_len, "Index is out of range!");
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.to_json()
    }

    pub fn get_total_kickstarters(&self) -> u32 {
        return self.kickstarters.len() as u32;
    }

    pub fn get_kickstarter_id_from_slug(&self, slug: String) -> KickstarterId {
        match self.kickstarter_id_by_slug.get(&slug) {
            Some(id) => id,
            None => panic!("Nonexistent slug!"),
        }
    }

    pub fn get_kickstarter_total_goals(&self, kickstarter_id: KickstarterIdJSON) -> u8 {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.get_number_of_goals()
    }

    pub fn get_kickstarter_goal(
        &self,
        kickstarter_id: KickstarterIdJSON,
        goal_id: GoalIdJSON,
    ) -> GoalJSON {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let goal = kickstarter.get_goal_by_id(goal_id.into());
        goal.to_json()
    }

    pub fn get_supporter_total_deposit_in_kickstarter(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
    ) -> BalanceJSON {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let deposit = kickstarter.get_deposit(&supporter_id);
        let result = match kickstarter.successful {
            Some(true) => {
                if kickstarter.stnear_price_at_unfreeze.is_some() {
                    let price_at_freeze = kickstarter.stnear_price_at_freeze
                        .expect("Price at freeze is not defined!");
                    let price_at_unfreeze = kickstarter.stnear_price_at_unfreeze
                        .expect("Price at unfreeze is not defined!");
                    proportional(deposit, price_at_freeze, price_at_unfreeze)
                } else {
                    panic!("Run get_supporter_estimated_stnear fn, to get an estimation while the funds are freezed!")
                }
            },
            _ => deposit,
        };
        BalanceJSON::from(result) 
    }

    pub fn get_supporter_estimated_stnear(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
        st_near_price: BalanceJSON,
    ) -> BalanceJSON {
        let supporter_id = SupporterId::from(supporter_id);
        let st_near_price = Balance::from(st_near_price);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful == Some(true) && kickstarter.stnear_price_at_unfreeze.is_none() {
            let price_at_freeze = kickstarter.stnear_price_at_freeze.unwrap();
            let deposit = kickstarter.get_deposit(&supporter_id);
            assert!(st_near_price >= price_at_freeze, "Please check the st_near_price you sent.");
            return BalanceJSON::from(proportional(deposit, price_at_freeze, st_near_price));
        } else {
            panic!("Run this fn only if the kickstarter has freezed funds.");
        }
    }

    pub fn get_supported_projects(&self, supporter_id: SupporterIdJSON) -> Vec<KickstarterIdJSON> {
        let supporter = self.internal_get_supporter(&supporter_id.into());
        supporter.supported_projects.to_vec()
    }

    pub fn get_supported_detailed_list(
        &self,
        supporter_id: SupporterIdJSON,
        from_index: u32,
        limit: u32,
    ) -> Option<Vec<SupporterDetailedJSON>> {
        // >>>> This FN is not working properly!

        let kickstarter_ids = self.get_supported_projects(supporter_id.clone());
        let kickstarters_len = kickstarter_ids.len() as u64;
        let start: u64 = from_index.into();
        if start > kickstarters_len {
            return None;
        }
        let mut result = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, kickstarters_len) {
            let kickstarter_id = kickstarter_ids.get(index as usize).unwrap();
            let kickstarter = self.internal_get_kickstarter(*kickstarter_id);
            let kickstarter_id = kickstarter.id;
            result.push(
                SupporterDetailedJSON {
                    kickstarter_id: KickstarterIdJSON::from(kickstarter_id),
                    supporter_deposit: self.get_supporter_total_deposit_in_kickstarter(supporter_id.clone(), kickstarter_id),
                    active: kickstarter.active,
                    successful: kickstarter.successful,
                }
            );
        }
        Some(result)
    }
}

/*************/
/*   Tests   */
/*************/

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, MockedBlockchain, VMContext};
    mod unit_test_utils;
    use unit_test_utils::*;
    use super::*;

    /// Get initial context for tests
    fn basic_context() -> VMContext {
        get_context(
            SYSTEM_ACCOUNT.into(),
            ntoy(TEST_INITIAL_BALANCE),
            0,
            to_ts(START_TIME_IN_DAYS),
            false,
        )
    }

    /// Creates a new contract
    fn new_contract() -> KatherineFundraising {
        KatherineFundraising::new(
            OWNER_ACCOUNT.into(),
            2,
            METAPOOL_CONTRACT_ADDRESS.to_string(),
            2,
        )
    }

    fn contract_only_setup() -> (VMContext, KatherineFundraising) {
        let context = basic_context();
        testing_env!(context.clone());
        let contract = new_contract();
        return (context, contract);
    }

    #[test]
    fn test_create_kickstarter() {
        let (_context, mut contract) = contract_only_setup();
        _new_kickstarter(_context, &mut contract);
        assert_eq!(1, contract.kickstarters.len());
    }

    #[test]
    fn test_get_kickstarters() {
        let (_context, mut contract) = contract_only_setup();
        contract.get_kickstarters(0, 49);
    }

    #[test]
    fn test_create_supporter() {
        let (_context, mut contract) = contract_only_setup();
        _new_kickstarter(_context, &mut contract);
        let kickstarter_id = contract.kickstarters.len() - 1;
        let mut k = contract.kickstarters.get(kickstarter_id).unwrap();
        k.update_supporter_deposits(&String::from(SUPPORTER_ACCOUNT), &DEPOSIT_AMOUNT);
        assert_eq!(1, k.get_total_supporters());
    }

    #[test]
    fn test_workflow() {
        let (_context, mut contract) = contract_only_setup();
        _new_kickstarter(_context, &mut contract);
        let kickstarter_id = contract.kickstarters.len() - 1;
        let mut k = contract.kickstarters.get(kickstarter_id).unwrap();
        k.update_supporter_deposits(&String::from(SUPPORTER_ACCOUNT), &DEPOSIT_AMOUNT);
        contract.create_goal(
            k.id,
            "test_goal".to_string(),
            U128::from(100),
            to_ts(START_TIME_IN_DAYS * 30), // WIP agregue para que compile
            U128::from(200),
            to_ts(START_TIME_IN_DAYS * 30),
            to_ts(START_TIME_IN_DAYS * 50),
        );
        contract.withdraw(U128::from(50), k.id);
    }
}
