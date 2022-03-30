use near_sdk::Gas;

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_MILLI_NEAR: u128 = NEAR / 1_000;

pub const BASIS_POINTS: u128 = 10_000;
pub const NO_DEPOSIT: u128 = 0;

/// Amount of gas for fungible token transfers.
pub const GAS: Gas = 20_000_000_000_000;
pub const TGAS: Gas = 1_000_000_000_000;
pub const FIVE_TGAS: Gas = 5 * TGAS;
pub const GAS_FOR_FT_TRANSFER: Gas = 47 * TGAS;
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 11 * TGAS;
pub const GAS_FOR_GET_STNEAR : Gas = 10 * TGAS;
