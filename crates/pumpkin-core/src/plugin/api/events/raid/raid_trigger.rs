use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a raid is triggered.
#[cancellable]
#[derive(Event, Clone)]
pub struct RaidTriggerEvent {
    /// Raid center block position.
    pub pos: BlockPos,
}

impl RaidTriggerEvent {
    #[must_use]
    pub const fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            cancelled: false,
        }
    }
}
