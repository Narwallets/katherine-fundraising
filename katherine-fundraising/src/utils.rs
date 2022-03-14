use near_sdk::{Balance, env};

use crate::types::*;
use crate::constants::*;

/// is_close returns true if total-0.001N < requested < total+0.001N
/// it is used to avoid leaving "dust" in the accounts and to manage rounding simplification for the users
/// e.g.: The user has 999999952342335499220000001 yN => 99.9999952342335499220000001 N
/// the UI shows 5 decimals rounded, so the UI shows "100 N". If the user chooses to liquid_unstake 100 N
/// the contract should take 100 N as meaning "all my tokens", and it will do because:
/// 99.9999952342335499220000001-0.001 < 100 < 99.9999952342335499220000001+0.001
#[inline]
pub fn is_close(requested: Balance, total: Balance) -> bool {
    requested >= total.saturating_sub(ONE_MILLI_NEAR) && requested <= total + ONE_MILLI_NEAR
}
#[inline]
pub fn get_epoch_millis() -> EpochMillis {
    return env::block_timestamp() / SECOND;
}
#[inline]
/// returns amount * numerator/denominator
pub fn proportional(amount: u128, numerator: u128, denominator: u128) -> u128 {
    return (U256::from(amount) * U256::from(numerator) / U256::from(denominator)).as_u128();
}
#[inline]
pub(crate) fn only_admin(&self, account: AccountId){
    assert!(env::predecessor_account_id() == self.owner_id, "only allowed for admin");
}
#[inline]
pub(crate) fn only_kickstarter_admin(&self, kickstarter: &Kickstarter){
    assert!(env::predecessor_account_id() == kickstarter.owner_id, "only allowed for kickstarter owner");
}