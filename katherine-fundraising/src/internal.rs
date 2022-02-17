use crate::*;
use near_sdk::{log, AccountId};
use near_sdk::Promise;
use near_sdk::serde_json::{json};

pub use crate::types::*;

impl KatherineFundraising {
    pub fn assert_min_deposit_amount(&self, amount: u128) {
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
    pub(crate) fn internal_deposit(&mut self) {
        self.assert_min_deposit_amount(env::attached_deposit());
        self.internal_deposit_attached_near_into(env::predecessor_account_id());
    }

    pub(crate) fn internal_deposit_attached_near_into(&mut self, account_id: AccountId) {
        let amount = env::attached_deposit();
        let mut account = self.internal_get_account(&account_id);

        account.available += amount;
        self.total_available += amount;

        self.internal_update_account(&account_id, &account);

        log!(
            "{} deposited into @{}'s account. New available balance is {}",
            amount,
            account_id,
            account.available
        );
    }

    /// Inner method to get the given account or a new default value account.
    pub(crate) fn internal_get_account(&self, account_id: &AccountId) -> Account {
        self.accounts.get(account_id).unwrap_or_default()
    }

    /// Inner method to save the given account for a given account ID.
    /// If the account balances are 0, the account is deleted instead to release storage.
    pub(crate) fn internal_update_account(&mut self, account_id: &AccountId, account: &Account) {
        if account.is_empty() {
            self.accounts.remove(account_id);
        } else {
            self.accounts.insert(account_id, &account); //insert_or_update
        }
    }

    pub(crate) fn transfer_back_to_account(&mut self, account_id: &AccountId, account: &mut Account) {
        let available: Balance = account.available;
        Promise::new(account_id.to_string()).transfer(available);
        account.available -= available;
        self.total_available -= available;
        self.internal_update_account(&account_id, &account);
    }

    pub(crate) fn internal_stake_funds(&mut self) {
        let metapool: AccountId = String::from("meta-v2.pool.testnet");
        Promise::new(metapool.to_string()).function_call(
            b"deposit_and_stake".to_vec(),
            json!({}).to_string().as_bytes().to_vec(),
            self.total_available,
            GAS,
        );
    }

    pub(crate) fn internal_withdraw(&mut self, requested_amount: Balance) -> Promise {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);

        let amount = account.take_from_available(requested_amount, self);
        self.internal_update_account(&account_id, &account);
        Promise::new(account_id)
    }
}