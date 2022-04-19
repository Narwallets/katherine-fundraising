use near_sdk::AccountId;
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GetAccountInfoResult {
    pub account_id: AccountId,
    pub available: U128,
    pub st_near: U128,  // Balance of st_near in the dummy contract
    pub valued_st_near: U128,
    pub meta: U128,
    pub realized_meta: U128,
    pub unstaked: U128,
    pub unstaked_requested_unlock_epoch: U64,
    pub unstake_full_epochs_wait_left: u16,
    pub can_withdraw: bool,
    pub total: U128,
    pub trip_start: U64,
    pub trip_start_stnear: U128,
    pub trip_accum_stakes: U128,
    pub trip_accum_unstakes: U128,
    pub trip_rewards: U128,
    pub nslp_shares: U128,
    pub nslp_share_value: U128,
    pub nslp_share_bp: u16,
}