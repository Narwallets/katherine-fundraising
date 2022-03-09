use near_sdk::{Gas};

pub const SECOND: u64 = 1_000_000_000;
pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;
pub const GAS: Gas = 20_000_000_000_000;
/// Amount of gas for fungible token transfers.
pub const TGAS: Gas = 1_000_000_000_000;
pub const GAS_FOR_FT_TRANSFER: Gas = 10 * TGAS;