use crate::*;

use near_sdk::{env, Balance};
use near_sdk::json_types::ValidAccountId;

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
pub fn get_current_epoch_millis() -> EpochMillis {
    return env::block_timestamp() / 1_000_000;
}

#[inline]
/// returns amount * numerator/denominator
pub fn proportional(amount: u128, numerator: u128, denominator: u128) -> u128 {
    return (U256::from(amount) * U256::from(numerator) / U256::from(denominator)).as_u128();
}

/// DEPRECATED: fn to calculate the release with steps. But the release will be full linear.
pub fn proportional_with_steps(
    amount: Balance,
    numerator: u128,
    denominator: u128,
    steps: u128,
) -> Balance {
    let mut amount_to_release: Balance = 0;
    let result = proportional(amount, numerator, denominator);
    for index in 1..steps {
        let proportion = proportional(amount, index, steps);
        if proportion <= result {
            amount_to_release = proportion;
        } else {
            break;
        }
    }
    amount_to_release
}

pub fn get_linear_release_proportion(
    amount: Balance,
    cliff_timestamp: EpochMillis,
    end_timestamp: EpochMillis,
) -> Balance {
    let now = get_current_epoch_millis();
    if now < cliff_timestamp {
        0
    } else if now >= end_timestamp {
        amount
    } else {
        let numerator = now as u128 - cliff_timestamp as u128;
        let denominator = end_timestamp as u128 - cliff_timestamp as u128;
        proportional(amount, numerator, denominator)
    }
}

pub fn convert_to_valid_account_id(account_id: AccountId) -> ValidAccountId {
    near_sdk::serde_json::from_str(&format!("\"{}\"", account_id)).unwrap()
}
