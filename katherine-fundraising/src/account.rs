use crate::*;

pub use crate::types::*;
pub use crate::utils::*;

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct Account {
    pub ready_to_fund: Balance,
    pub locked: Balance,
    pub available: u128,
}

/// User account on this contract
impl Default for Account {
    fn default() -> Self {
        Self {
            available: 0,
        }
    }
}
impl Account {
    /// when the account.is_empty() it will be removed
    pub fn is_empty(&self) -> bool {
        return self.available == 0;
    }

    pub(crate) fn take_from_available(
        &mut self,
        amount_requested: Balance,
        main: &mut KatherineFundraising,
    ) -> Balance {
        let to_withdraw:Balance =
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
            "Not enough available balance {} for the requested amount",
            self.available
        );
        self.available -= to_withdraw;

        assert!(main.total_available >= to_withdraw, "i_s_Inconsistency");
        main.total_available -= to_withdraw;

        return to_withdraw;
    }
}