use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Supporter {
    pub supported_projects: UnorderedSet<KickstarterId>,
}

impl Supporter {
    pub fn new(id: &SupporterId) -> Self {
        Self {
            supported_projects: UnorderedSet::new(Keys::SupportedProjects.as_prefix(&id).as_bytes()),
        }
    }

    pub fn is_supporting(&self, kickstarter_id: KickstarterId) -> bool {
        return self.supported_projects.contains(&kickstarter_id)
    }

    /// when the supporter.is_empty() it will be removed
    pub fn is_empty(&self) -> bool {
        return self.supported_projects.is_empty();
    }
}
