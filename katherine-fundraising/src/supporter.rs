use near_sdk::collections::{Vector};

use crate::*;


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Supporter {
    pub total_in_deposits: Balance,
    pub locked: Balance,
    pub available: u128,
    pub kickstarters: Vector<KickstarterId>
}

/// Supporter account on this contract
impl Default for Supporter {
    fn default() -> Self {
        Self {
            available: 0,
            total_in_deposits: 0,
            locked: 0,
            kickstarters: Vector::new(b"Kickstarter".to_vec()),
        }
    }
}
impl Supporter {
    /// when the supporter.is_empty() it will be removed
    pub fn is_empty(&self) -> bool {
        return self.available == 0
               && self.total_in_deposits == 0
               && self.locked == 0
               && self.kickstarters.is_empty();
    }

    pub(crate) fn take_from_available(
        &mut self,
        amount_requested: Balance,
        main: &mut KatherineFundraising,
    ) -> Balance {
        let to_withdraw: Balance =
        // if the amount is close to user's total, remove user's total
        // to: a) do not leave less than ONE_MILLI_NEAR in the account, b) Allow some yoctos of rounding, e.g. remove(100) removes 99.999993 without panicking
        // Audit Note: Do not do this for .lockup accounts because the lockup contract relies on precise amounts
        if !env::predecessor_account_id().ends_with(".lockup.near") && is_close(amount_requested, self.available) { // allow for rounding simplification
            self.available
        }
        else {
            amount_requested
        };

        assert!(
            self.available >= to_withdraw,
            "Not enough available balance {} for the requested supporter",
            self.available
        );
        self.available -= to_withdraw;

        assert!(main.total_available >= to_withdraw, "i_s_Inconsistency");
        main.total_available -= to_withdraw;

        return to_withdraw;
    }
}