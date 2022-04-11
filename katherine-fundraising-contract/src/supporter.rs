use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Supporter {
    pub total_in_deposits: Balance,
    pub locked: Balance,
    pub available: u128,
    pub supported_projects: UnorderedSet<KickstarterId>,
}

/// Supporter account on this contract
impl Default for Supporter {
    fn default() -> Self {
        Self {
            available: 0,
            total_in_deposits: 0,
            locked: 0,
            supported_projects: UnorderedSet::new(b"Kickstarter".to_vec()),
        }
    }
}

impl Supporter {
    /// when the supporter.is_empty() it will be removed
    pub fn is_empty(&self) -> bool {
        return self.available == 0
            && self.total_in_deposits == 0
            && self.locked == 0
            && self.supported_projects.is_empty();
    }
}
