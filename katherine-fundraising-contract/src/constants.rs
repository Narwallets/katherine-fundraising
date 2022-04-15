use near_sdk::Gas;
use near_sdk::BorshIntoStorageKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

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

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Keys {
    KickstarterId,
    Supporters,
    Kickstarters,
    SupporterKickstarters,
    Active,
    Goals,
    Deposits,
    Withdraws,
}

impl Keys {
	/// Creates unique prefix for collections based on a u32 integer
	pub fn as_prefix(&self, id: &u32) -> String {
		match self {
			Keys::KickstarterId => format!("{}{}", "Kid", id),
			Keys::Supporters => format!("{}{}", "S", id),
			Keys::Kickstarters => format!("{}{}", "K", id),
			Keys::SupporterKickstarters => format!("{}{}", "Sk", id),
			Keys::Active => format!("{}{}", "A", id),
			Keys::Goals => format!("{}{}", "G", id),
			Keys::Deposits => format!("{}{}", "D", id),
			Keys::Withdraws => format!("{}{}", "W", id),
		}
    }
}

impl BorshIntoStorageKey for Keys {}
