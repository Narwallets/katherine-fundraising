use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult};

mod claim;
mod constants;
mod deposit;
mod interest;
mod internal;
mod types;
mod withdraw;

pub mod goal;
pub mod interface;
pub mod kickstarter;
pub mod supporter;
pub mod utils;
pub use crate::utils::*;

use crate::{constants::*, goal::*, kickstarter::*, supporter::*, types::*};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineFundraising {
    pub owner_id: AccountId,
    pub supporters: UnorderedMap<SupporterId, Supporter>,
    pub kickstarters: Vector<Kickstarter>,
    pub kickstarter_id_by_slug: UnorderedMap<String, KickstarterId>,

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
        assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            supporters: UnorderedMap::new(Keys::Supporters),
            kickstarters: Vector::new(Keys::Kickstarters),
            kickstarter_id_by_slug: UnorderedMap::new(Keys::KickstarterId),
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
                    }
                    None => {
                        kickstarter.active = false;
                        self.active_projects.remove(&kickstarter.id);
                        kickstarter.successful = Some(false);
                        self.kickstarters
                            .replace(kickstarter_id as u64, &kickstarter);
                        log!("kickstarter successfully deactivated");
                    }
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
            if kickstarter.successful == Some(true) && kickstarter.stnear_price_at_unfreeze == None
            {
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
            log!(
                "UNFREEZE: funds successfully unfreezed for Kickstarter {}",
                kickstarter_id
            );
        }
    }

    /*****************************/
    /*   Supporters functions    */
    /*****************************/

    pub fn withdraw_all(&mut self, kickstarter_id: KickstarterIdJSON) {
        let supporter_id = convert_to_valid_account_id(env::predecessor_account_id());
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if !kickstarter.is_within_funding_period() {
            kickstarter.assert_funds_must_be_unfreezed();
        }
        let amount = self.get_supporter_total_deposit_in_kickstarter(supporter_id, kickstarter_id, None);
        self.withdraw(amount, kickstarter_id);
    }

    /// Withdraw a valid amount of user's balance. Call this before or after the Locking Period.
    pub fn withdraw(&mut self, amount: BalanceJSON, kickstarter_id: KickstarterIdJSON) {
        let min_prepaid_gas = GAS_FOR_FT_TRANSFER + GAS_FOR_RESOLVE_TRANSFER + FIVE_TGAS;
        assert!(
            env::prepaid_gas() > min_prepaid_gas,
            "gas required {}",
            min_prepaid_gas
        );
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let amount = Balance::from(amount);
        assert!(
            amount > 0,
            "The amount to withdraw should be greater than Zero!"
        );
        let supporter_id: SupporterId = env::predecessor_account_id();
        match kickstarter.successful {
            Some(true) => {
                kickstarter.assert_funds_must_be_unfreezed();
                self.internal_supporter_withdraw_after_unfreeze(
                    amount,
                    &mut kickstarter,
                    supporter_id,
                );
            }
            Some(false) => {
                self.internal_supporter_withdraw_before_freeze(
                    amount,
                    &mut kickstarter,
                    supporter_id,
                );
            }
            None => {
                assert!(
                    get_current_epoch_millis() < kickstarter.close_timestamp,
                    "The funding period is over, Kickstarter must be evaluated!"
                );
                self.internal_supporter_withdraw_before_freeze(
                    amount,
                    &mut kickstarter,
                    supporter_id,
                );
            }
        };
    }

    pub fn claim_all_kickstarter_tokens(&mut self, kickstarter_id: KickstarterIdJSON) {
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
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        self.internal_claim_kickstarter_tokens(amount, &mut kickstarter, account_id);
    }

    /*****************************/
    /*   Kickstarter functions   */
    /*****************************/

    pub fn withdraw_stnear_interest(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
    ) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.assert_kickstarter_owner();
        assert_eq!(
            kickstarter.successful,
            Some(true),
            "Kickstarter is unsuccessful!"
        );

        let receiver_id = env::predecessor_account_id();
        if let Some(st_near_price) = kickstarter.stnear_price_at_unfreeze {
            // No need to get stnear price from metapool.
            self.kickstarter_withdraw(&mut kickstarter, st_near_price, receiver_id);
        } else {
            self.kickstarter_withdraw_before_unfreeze(&mut kickstarter, receiver_id);
        }
    }

    pub fn kickstarter_withdraw_excedent(&mut self, kickstarter_id: KickstarterIdJSON) {
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        kickstarter.assert_kickstarter_owner();
        assert!(
            kickstarter.close_timestamp < get_current_epoch_millis(),
            "The excedent is avalable only after the funding period ends"
        );

        let excedent: Balance = match kickstarter.successful {
            Some(true) => {
                let katherine_fee = kickstarter.katherine_fee.unwrap();
                let total_tokens_to_release = kickstarter.total_tokens_to_release.unwrap();
                kickstarter.available_reward_tokens - (katherine_fee + total_tokens_to_release)
            }
            Some(false) => {
                log!("Returning all available reward tokens!");
                kickstarter.available_reward_tokens
            }
            None => panic!(
                "Before withdrawing pTOKEN, evaluate the project using the process_kickstarter fn!"
            ),
        };

        if excedent > 0 {
            self.internal_withdraw_excedent(&mut kickstarter, excedent);
        } else {
            panic!("No remaining excedent pTOKEN to withdraw!");
        }
    }

    /***********************/
    /*   Admin functions   */
    /***********************/

    /// Withdraws the Katherine Fee from a Kickstarter.
    pub fn withdraw_katherine_fee(&mut self, kickstarter_id: KickstarterIdJSON) {
        self.assert_only_admin();
        let mut kickstarter = self.internal_get_kickstarter(kickstarter_id);
        assert!(
            kickstarter.close_timestamp < get_current_epoch_millis(),
            "To withdraw the Katherine Fee the Kickstarter must be closed."
        );
        let katherine_fee: Balance = if kickstarter.successful == Some(true) {
            kickstarter.katherine_fee.unwrap().into()
        } else {
            panic!("Kickstarter was unsuccessful.");
        };

        if katherine_fee > 0 {
            self.internal_withdraw_katherine_fee(&mut kickstarter, katherine_fee);
        } else {
            panic!("Katherine fee is 0.");
        }
    }

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
        self.internal_create_kickstarter(
            id,
            name,
            slug,
            owner_id,
            open_timestamp,
            close_timestamp,
            token_contract_address,
            deposits_hard_cap,
            max_tokens_to_release_per_stnear
        )
    }

    pub fn delete_kickstarter(&mut self, id: KickstarterId) {
        panic!("Kickstarter {} must not be deleted!", id);
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
        self.internal_update_kickstarter(
            id,
            name,
            slug,
            owner_id,
            open_timestamp,
            close_timestamp,
            token_contract_address,
            deposits_hard_cap,
            max_tokens_to_release_per_stnear
        );
    }

    pub fn create_goal(
        &mut self,
        kickstarter_id: KickstarterId,
        name: String,
        desired_amount: BalanceJSON,
        unfreeze_timestamp: EpochMillis,
        tokens_to_release_per_stnear: BalanceJSON,
        cliff_timestamp: EpochMillis,
        end_timestamp: EpochMillis,
    ) -> GoalId {
        self.internal_create_goal(
            kickstarter_id,
            name,
            desired_amount,
            unfreeze_timestamp,
            tokens_to_release_per_stnear,
            cliff_timestamp,
            end_timestamp,
        )
    }

    pub fn delete_last_goal(
        &mut self,
        kickstarter_id: KickstarterId
    ) {
        self.internal_delete_last_goal(kickstarter_id);
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
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
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
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        match self.supporters.get(&supporter_id) {
            Some(supporter) => {
                if supporter.is_supporting(kickstarter.id) && kickstarter.winner_goal_id.is_some() {
                    let rewards = self.internal_get_available_rewards(&supporter_id, &kickstarter);
                    return Some(BalanceJSON::from(rewards));
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }

    /// Available rewards that the Supporter could currently claim.
    pub fn get_admin_fee_rewards(
        &self,
        kickstarter_id: KickstarterIdJSON,
    ) -> BalanceJSON {
        let kickstarter = self.internal_get_kickstarter(kickstarter_id);
        if kickstarter.successful == Some(true) {
            kickstarter.katherine_fee.unwrap().into()
        } else {
            panic!("Kickstarter was unsuccessful.");
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
                        kickstarter_id,
                        st_near_price,
                    );
                }
            }
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
            assert!(
                st_near_price >= price_at_freeze,
                "Please check the st_near_price you sent."
            );
            let amount = kickstarter.get_deposit(&supporter_id);
            // No need to review stnear_withdraw because funds are still freezed.
            BalanceJSON::from(proportional(amount, price_at_freeze, st_near_price))
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
            let supporter_deposit = self.get_supporter_total_deposit_in_kickstarter(
                supporter_id.clone(),
                kickstarter_id,
                Some(st_near_price)
            );
            let deposit_in_near = kickstarter.get_at_freeze_deposits_in_near(
                &supporter_id.to_string()
            );
            let rewards = self.get_supporter_total_rewards(
                supporter_id.clone(),
                kickstarter_id
            );
            let available_rewards = self.get_supporter_available_rewards(
                supporter_id.clone(),
                kickstarter_id
            );
            result.push(
                SupporterDetailedJSON {
                    kickstarter_id,
                    supporter_deposit,
                    deposit_in_near,
                    rewards,
                    available_rewards,
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
    use super::*;
    use unit_test_utils::*;

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
