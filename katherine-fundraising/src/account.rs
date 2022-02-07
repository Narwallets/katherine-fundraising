use crate::*;

// pub use crate::types::*;
// pub use crate::utils::*;

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct Account {
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
}