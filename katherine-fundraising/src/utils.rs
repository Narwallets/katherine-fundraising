use near_sdk::{Balance, env};

use crate::{constants::*, types::*};

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

pub fn get_epoch_millis() -> EpochMillis {
    return env::block_timestamp() / SECOND;
}