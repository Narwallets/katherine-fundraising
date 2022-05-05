use near_sdk::Gas;
use near_sdk::BorshIntoStorageKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::types::SupporterId;

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;

pub const BASIS_POINTS: u128 = 10_000;
pub const NO_DEPOSIT: u128 = 0;

/// Amount of gas for fungible token transfers.
pub const TGAS: Gas = 1_000_000_000_000;
pub const FIVE_TGAS: Gas = 5 * TGAS;
pub const GAS_FOR_FT_TRANSFER: Gas = 47 * TGAS;
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 11 * TGAS;
pub const GAS_FOR_GET_STNEAR : Gas = 10 * TGAS;
pub const GAS_FOR_INTEREST_WITHDRAW : Gas = 200 * TGAS;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Keys {
    KickstarterId,
    Supporters,
    Kickstarters,
    SupportedProjects,
    Active,
    Goals,
    Deposits,
    RewardWithdraws,
    StnearWithdraws,
}

impl Keys {
	/// Creates unique prefix for collections based on a String id.
	pub fn as_prefix(&self, id: &str) -> String {
		match self {
			Keys::KickstarterId => format!("{}{}", "Kid", id),
			Keys::Supporters => format!("{}{}", "S", id),
			Keys::Kickstarters => format!("{}{}", "K", id),
			Keys::SupportedProjects => format!("{}{}", "Sp", id),
			Keys::Active => format!("{}{}", "A", id),
			Keys::Goals => format!("{}{}", "G", id),
			Keys::Deposits => format!("{}{}", "D", id),
			Keys::RewardWithdraws => format!("{}{}", "RW", id),
			Keys::StnearWithdraws => format!("{}{}", "SW", id),
		}
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum WithdrawEntity {
    Kickstarter,
    Supporter(SupporterId),
}

impl BorshIntoStorageKey for Keys {}
impl BorshIntoStorageKey for WithdrawEntity {}
