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
    pub max_reward_installments: u32,

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
            max_reward_installments: 12,
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

    /// Returns kickstarters ids ready to unfreeze.
    pub fn get_kickstarters_to_unfreeze(
        &self,
        from_index: KickstarterIdJSON,
        limit: KickstarterIdJSON,
    ) -> Option<Vec<KickstarterIdJSON>> {
        let kickstarters_len = self.kickstarters.len();
        let start: u64 = from_index.into();
        if start >= kickstarters_len {
            return None;
        }
        let mut result: Vec<KickstarterIdJSON> = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, kickstarters_len) {
            let kickstarter = self.internal_get_kickstarter(index as u32);
            if kickstarter.successful == Some(true) && kickstarter.stnear_price_at_unfreeze == None {
                if kickstarter.funds_can_be_unfreezed() {
                    result.push(KickstarterIdJSON::from(kickstarter.id));
                }
            }
        }
        Some(result)
    }

    /// Start the cross-contract call to unfreeze the kickstarter funds.
    pub fn unfreeze_kickstarter_funds(&mut self, kickstarter_id: KickstarterIdJSON) {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful == Some(true) && kickstarter.stnear_price_at_unfreeze == None {
            kickstarter.assert_funds_can_be_unfreezed();
            self.internal_unfreeze_kickstarter_funds(kickstarter_id);
            log!("UNFREEZE: funds successfully unfreezed for Kickstarter {}", kickstarter_id);
        }
    }

    /*****************************/
    /*   Supporters functions    */
    /*****************************/

    pub fn withdraw_all(&mut self, kickstarter_id: KickstarterIdJSON) {
        let supporter_id = convert_to_valid_account_id(env::predecessor_account_id());
        let amount = self.get_supporter_total_deposit_in_kickstarter(supporter_id, kickstarter_id, None);
        self.withdraw(amount, kickstarter_id);
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let min_prepaid_gas = GAS_FOR_FT_TRANSFER + GAS_FOR_RESOLVE_TRANSFER + FIVE_TGAS;
        assert!(env::prepaid_gas() > min_prepaid_gas, "gas required {}", min_prepaid_gas);
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        let amount = Balance::from(amount);
        assert!(amount > 0, "The amount to withdraw should be greater than Zero!");
        let supporter_id: SupporterId = env::predecessor_account_id();
        match kickstarter.successful {
            Some(true) => {
                kickstarter.assert_funds_must_be_unfreezed();
                self.internal_supporter_withdraw_after_unfreeze(
                    amount,
                    &mut kickstarter,
                    supporter_id
                );
            },
            Some(false) => {
                self.internal_supporter_withdraw_before_freeze(
                    amount,
                    &mut kickstarter,
                    supporter_id
                );
            },
            None => {
                assert!(
                    get_current_epoch_millis() < kickstarter.close_timestamp,
                    "The funding period is over, Kickstarter must be evaluated!"
                );
                self.internal_supporter_withdraw_before_freeze(
                    amount,
                    &mut kickstarter,
                    supporter_id
                );
            },
        };
    }

    pub fn claim_all_kickstarter_tokens(
        &mut self,
        kickstarter_id: KickstarterIdJSON
    ) {
        let account_id = env::predecessor_account_id();
        let available_rewards = self.get_supporter_available_rewards(
            convert_to_valid_account_id(account_id),
            kickstarter_id,
        );
        if let Some(amount) = available_rewards {
            self.claim_kickstarter_tokens(amount, kickstarter_id);
        } else {
            panic!("Supporter does not have available Kickstarter Tokens");
        }
    }

    // lets supporters withdraw the tokens emited by the kickstarter
    pub fn claim_kickstarter_tokens(
        &mut self,
        amount: BalanceJSON,
        kickstarter_id: KickstarterIdJSON,
    ) {
        let account_id = env::predecessor_account_id();
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        self.internal_claim_kickstarter_tokens(
            amount,
            &mut kickstarter,
            account_id
        );
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
            log!(
                "WITHDRAW: {} pTOKEN withdraw from KickstarterId {} to Account {}",
                excedent,
                kickstarter_id,
                kickstarter.owner_id,
            );
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
            panic!("No remaining excedent pTOKEN to withdraw!");
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

    pub(crate) fn internal_restore_kickstarter_excedent_withdraw(
        &mut self,
        amount: Balance,
        kickstarter_id: KickstarterId,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.available_reward_tokens += amount;
        self.kickstarters
            .replace(kickstarter_id as u64, &kickstarter);
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
            id,
            name,
            slug,
            goals: Vector::new(Keys::Goals.as_prefix(&id.to_string()).as_bytes()),
            winner_goal_id: None,
            katherine_fee: None,
            total_tokens_to_release: None,
            deposits: UnorderedMap::new(Keys::Deposits.as_prefix(&id.to_string()).as_bytes()),
            rewards_withdraw: UnorderedMap::new(Keys::RewardWithdraws.as_prefix(&id.to_string()).as_bytes()),
            stnear_withdraw: UnorderedMap::new(Keys::StnearWithdraws.as_prefix(&id.to_string()).as_bytes()),
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
            goals: Vector::new(Keys::Goals.as_prefix(&id.to_string()).as_bytes()),
            winner_goal_id: None,
            katherine_fee: None,
            total_tokens_to_release: None,
            deposits: UnorderedMap::new(Keys::Deposits.as_prefix(&id.to_string()).as_bytes()),
            rewards_withdraw: UnorderedMap::new(Keys::RewardWithdraws.as_prefix(&id.to_string()).as_bytes()),
            stnear_withdraw: UnorderedMap::new(Keys::StnearWithdraws.as_prefix(&id.to_string()).as_bytes()),
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

    /// Get the total rewards that the Supporter could claim regardless of the current timestamp.
    pub fn get_supporter_total_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
    ) -> Option<BalanceJSON> {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.is_supporting(kickstarter.id) && kickstarter.winner_goal_id.is_some() {
                    let goal = kickstarter.get_winner_goal();
                    let rewards = self.internal_get_supporter_rewards(
                        &supporter_id,
                        &kickstarter,
                        goal.tokens_to_release_per_stnear,
                    );
                    return Some(BalanceJSON::from(rewards));
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }

    /// Available rewards that the Supporter could currently claim.
    pub fn get_supporter_available_rewards(
        &self,
        supporter_id: SupporterIdJSON,
        kickstarter_id: KickstarterIdJSON,
    ) -> Option<BalanceJSON> {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id.into());
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.is_supporting(kickstarter.id) && kickstarter.winner_goal_id.is_some() {
                    let rewards = self.internal_get_available_rewards(
                        &supporter_id,
                        &kickstarter,
                    );
                    return Some(BalanceJSON::from(rewards));
                } else {
                    return None;
                }
            }
            None => return None,
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
        st_near_price: Option<BalanceJSON>,
    ) -> BalanceJSON {
        let supporter_id = SupporterId::from(supporter_id);
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let result = match kickstarter.successful {
            Some(true) => {
                if kickstarter.is_unfreeze() {
                    let entity = WithdrawEntity::Supporter(supporter_id.to_string());
                    kickstarter.get_after_unfreeze_deposits(&supporter_id)
                        - kickstarter.get_stnear_withdraw(&entity)
                } else {
                    let st_near_price = st_near_price
                        .expect("An exact value is not available. Please send the current stNEAR price to calculate an estimation");
                    return self.get_supporter_estimated_stnear(
                        convert_to_valid_account_id(supporter_id),
                        kickstarter_id.into(),
                        st_near_price
                    );
                }
            },
            _ => kickstarter.get_deposit(&supporter_id),
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
            assert!(st_near_price >= price_at_freeze, "Please check the st_near_price you sent.");
            let amount = kickstarter.get_deposit(&supporter_id);
            // No need to review stnear_withdraw because funds are still freezed.
            BalanceJSON::from(
                proportional(
                    amount,
                    price_at_freeze,
                    st_near_price
                )
            )
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
        st_near_price: BalanceJSON,
        from_index: u32,
        limit: u32,
    ) -> Option<Vec<SupporterDetailedJSON>> {
        let kickstarter_ids = self.get_supported_projects(supporter_id.clone());
        let kickstarters_len = kickstarter_ids.len() as u64;
        let start: u64 = from_index.into();
        if start > kickstarters_len || kickstarters_len == 0 {
            return None;
        }
        let mut result = Vec::new();
        for index in start..std::cmp::min(start + limit as u64, kickstarters_len) {
            let kickstarter_id = *kickstarter_ids.get(index as usize).unwrap();
            let kickstarter = self.internal_get_kickstarter(kickstarter_id);
            result.push(
                SupporterDetailedJSON {
                    kickstarter_id,
                    supporter_deposit: self.get_supporter_total_deposit_in_kickstarter(
                        supporter_id.clone(),
                        kickstarter_id.into(),
                        Some(st_near_price)
                    ),
                    rewards: self.get_supporter_total_rewards(
                        supporter_id.clone(),
                        kickstarter_id.into()
                    ),
                    available_rewards: self.get_supporter_available_rewards(
                        supporter_id.clone(),
                        kickstarter_id.into()
                    ),
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
    use near_sdk::json_types::{U128, ValidAccountId};
    mod unit_test_utils;
    use unit_test_utils::*;
    use super::*;
    use crate::constants::NEAR;
    use std::convert::TryFrom;

    /// Get initial context for tests
    fn basic_context() -> VMContext {
        get_context(
            SYSTEM_ACCOUNT.into(),
            ntoy(TEST_INITIAL_BALANCE),
            0,
            1_000_000_000,
            false,
        )
    }

    /// Creates a new contract
    fn new_contract() -> KatherineFundraising {
        KatherineFundraising::new(
            OWNER_ACCOUNT.into(),
            U128::from(2),
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
        _new_kickstarter(&_context, &mut contract, "Test Kickstarter".to_owned(),
            "test_kickstarter".to_owned(),
            get_time_millis(&_context),
            get_time_millis(&_context) + 1_000 * 60 * 5,
            );
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
        _new_kickstarter(&_context, &mut contract, "Test Kickstarter".to_owned(),
            "test_kickstarter".to_owned(),
            get_time_millis(&_context),
            get_time_millis(&_context) + 1_000 * 60 * 5,
        );
        let kickstarter_id = contract.kickstarters.len() - 1;
        let mut k = contract.kickstarters.get(kickstarter_id).unwrap();
        k.enough_reward_tokens = true;
        k.deposits_hard_cap = 100000;
        contract.internal_supporter_deposit(&String::from(SUPPORTER_ACCOUNT), &DEPOSIT_AMOUNT, &mut k);
        assert_eq!(1, k.get_total_supporters());
    }

    #[test]
    fn test_supported_projects() {
        use std::convert::TryInto;
        let (_context, mut contract) = contract_only_setup();
        // TODO: move this to a function
        // First kickstarter
        _new_kickstarter(&_context, &mut contract, "Fist kickstarter".to_owned(),
            "first_slig".to_owned(),
            get_time_millis(&_context),
            get_time_millis(&_context) + 1_000 * 60 * 5,
        );
        let kickstarter_id = contract.kickstarters.len() - 1;
        let mut k = contract.kickstarters.get(kickstarter_id).unwrap();
        k.enough_reward_tokens = true;
        k.deposits_hard_cap = 100000;
        println!("Second kickstarter...");
        // Second kickstarter
        let start = env::block_timestamp();
        _new_kickstarter(&_context, &mut contract, "Second kickstarter".to_owned(),
            "second_slug".to_owned(),
            get_time_millis(&_context),
            get_time_millis(&_context) + 1_000 * 60 * 5,
        );
        let kickstarter_id2 = contract.kickstarters.len() - 1;
        let mut k2 = contract.kickstarters.get(kickstarter_id2).unwrap();
        k2.enough_reward_tokens = true;
        k2.deposits_hard_cap = 100000;

        let s = String::from(SUPPORTER_ACCOUNT);
        contract.internal_supporter_deposit(&s, &DEPOSIT_AMOUNT, &mut k);
        contract.internal_supporter_deposit(&s, &DEPOSIT_AMOUNT, &mut k2);
        let kickstarters = contract.get_supported_detailed_list(s.try_into().unwrap(), U128::from(1), 0, 10);
        assert_eq!(2, kickstarters.unwrap().len());
    }


    #[test]
    fn test_workflow() {
        use std::convert::TryInto;
        let (mut _context, mut contract) = contract_only_setup();
        // TODO: move this to a function
        // First kickstarter
        _new_kickstarter(&_context, &mut contract, "Fist kickstarter".to_owned(),
            "first_slig".to_owned(),
            get_time_millis(&_context),
            get_time_millis(&_context) + 1_000 * 60 * 5,
        );
        let kickstarter_id = contract.kickstarters.len() - 1;
        let mut k = contract.kickstarters.get(kickstarter_id).unwrap();
        k.enough_reward_tokens = true;
        k.deposits_hard_cap = 100000;
        println!("Second kickstarter...");
        // Second kickstarter
        let start = get_time_millis(&_context);
        _new_kickstarter(&_context, &mut contract, "Second kickstarter".to_owned(),
            "second_slug".to_owned(),
            start,
            start + 1_000 * 60 * 5,
        );
        let kickstarter_id2 = contract.kickstarters.len() - 1;
        let mut k2 = contract.kickstarters.get(kickstarter_id2).unwrap();
        k2.enough_reward_tokens = true;
        k2.deposits_hard_cap = 100000;


		contract.create_goal(
	        k.id,
	        String::from("Goal 1"),
	        BalanceJSON::from(100),
	        start + 1_000 * 60 * 2,
	        BalanceJSON::from(200 * NEAR),
	        start + 1_000 * 60 * 3,
	        start + 1_000 * 60 * 4,
	        10,
		);

		contract.create_goal(
	        k.id,
	        String::from("Goal 2"),
	        BalanceJSON::from(200 * NEAR),
	        start + 1_000 * 60 * 2,
	        BalanceJSON::from(300 * NEAR),
	        start + 1_000 * 60 * 4,
	        start + 1_000 * 60 * 5,
	        10,
		);

		contract.create_goal(
	        k2.id,
	        String::from("Goal 1"),
	        BalanceJSON::from(400 * NEAR),
	        start + 1_000 * 60 * 2,
	        BalanceJSON::from(500 * NEAR),
	        start + 1_000 * 60 * 4,
	        start + 1_000 * 60 * 5,
	        10,
		);
        // This references might be wrong
        contract.internal_kickstarter_deposit(&(NEAR * 20), &mut k);
        _context.predecessor_account_id = SUPPORTER_ACCOUNT.to_owned();
        contract.withdraw(U128::from(NEAR), k.id.into());
        contract.withdraw_all(k.id.into());
        contract.internal_supporter_deposit(&SUPPORTER_ACCOUNT.to_owned(), &(200 * NEAR), &mut k);

        let supported_projects = contract.get_supported_detailed_list(
            ValidAccountId::try_from(SUPPORTER_ACCOUNT).unwrap(),
            U128::from(NEAR * 2), 1, 10);
        let kp = contract.get_kickstarters_to_process(0, 10);
        contract.process_kickstarter(k.id);
        contract.kickstarter_withdraw_excedent(k.id);

        let kickstarter_detail = contract.get_project_details(k.id);

        let s = String::from(SUPPORTER_ACCOUNT);
        contract.internal_supporter_deposit(&s, &DEPOSIT_AMOUNT, &mut k);
        contract.internal_supporter_deposit(&s, &DEPOSIT_AMOUNT, &mut k2);
    }
}
