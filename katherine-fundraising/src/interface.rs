use near_sdk::{ext_contract};
use near_sdk::json_types::{U128};


use crate::*;

#[ext_contract(nep141_token)]
pub trait NEP141Token {
    fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self_metapool)]
pub trait ExtSelfMetapool {
    fn return_tokens_callback(&mut self, user: AccountId, kickstarter_id: KickstarterIdJSON, amount: U128);
}

#[ext_contract(ext_self_kikstarter)]
pub trait ExtSelfKickstarter {
    fn return_tokens_from_kickstarter_callback(&mut self, user: AccountId, kickstarter_id: KickstarterIdJSON, amount: U128);
    fn kickstarter_withdraw_excedent_callback(&mut self, kickstarter_id: KickstarterIdJSON, amount: U128);
}
