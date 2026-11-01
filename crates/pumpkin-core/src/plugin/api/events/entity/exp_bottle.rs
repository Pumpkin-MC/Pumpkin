use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an experience bottle breaks.
#[cancellable]
#[derive(Event, Clone)]
pub struct ExpBottleEvent {
    /// Bottle entity ID.
    pub entity_id: i32,
    /// Experience amount.
    pub experience: i32,
    /// Location of the bottle break.
    pub location: BlockPos,
    /// Whether to show the particle effect.
    pub show_effect: bool,
}

impl ExpBottleEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        experience: i32,
        location: BlockPos,
        show_effect: bool,
    ) -> Self {
        Self {
            entity_id,
            experience,
            location,
            show_effect,
            cancelled: false,
        }
    }
}
