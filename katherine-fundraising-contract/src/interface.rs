use crate::*;

use near_sdk::ext_contract;
use near_sdk::json_types::{U128, ValidAccountId};

#[ext_contract(nep141_token)]
pub trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
    );
}

#[ext_contract(ext_self_metapool)]
pub trait ExtSelfMetapool {
    fn return_tokens_before_freeze_callback(
        &mut self,
        supporter_id: ValidAccountId,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    );

    fn return_tokens_after_unfreeze_callback(
        &mut self,
        supporter_id: ValidAccountId,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    );

    fn get_st_near_price(&self) -> U128String;
}

#[ext_contract(ext_self_kickstarter)]
pub trait ExtSelfKickstarter {
    fn activate_successful_kickstarter_after(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        goal_id: GoalIdJSON,
    );

    fn kickstarter_withdraw_excedent_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    );

    fn return_tokens_from_kickstarter_callback(
        &mut self,
        account_id: ValidAccountId,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    );

    fn kickstarter_withdraw_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        receiver_id: ValidAccountId,
    );

    fn kickstarter_withdraw_resolve_transfer(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
        receiver_id: ValidAccountId,
    );

    fn set_stnear_price_at_unfreeze(
        &mut self,
        kickstarter_id: KickstarterIdJSON
    );

    fn withdraw_kickstarter_fee_callback(
        &mut self,
        kickstarter_id: KickstarterIdJSON,
        amount: U128,
    )->Result;
    
}
