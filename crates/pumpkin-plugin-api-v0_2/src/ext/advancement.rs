use crate::wit::pumpkin::plugin::advancement::{AdvancementProgress, FrameType};

impl AdvancementProgress {
    /// Returns `true` if the advancement is fully completed.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.done
    }

    /// Checks if a specific criterion has been awarded.
    #[must_use]
    pub fn is_criterion_done(&self, criterion: &str) -> bool {
        self.awarded_criteria.iter().any(|c| c == criterion)
    }

    /// Returns a slice of the awarded criteria names.
    #[must_use]
    pub fn get_awarded_criteria(&self) -> &[String] {
        &self.awarded_criteria
    }

    /// Returns a slice of the remaining criteria names.
    #[must_use]
    pub fn get_remaining_criteria(&self) -> &[String] {
        &self.remaining_criteria
    }
}

impl FrameType {
    /// Returns the vanilla translation key suffix or identifier for this frame type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Challenge => "challenge",
            Self::Goal => "goal",
        }
    }
}
