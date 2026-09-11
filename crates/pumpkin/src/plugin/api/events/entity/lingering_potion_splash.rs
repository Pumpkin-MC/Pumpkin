use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a lingering potion splashes.
#[cancellable]
#[derive(Event, Clone)]
pub struct LingeringPotionSplashEvent {
    /// Potion entity ID.
    pub entity_id: i32,
    /// Location of the splash.
    pub location: BlockPos,
    /// Potion item name.
    pub potion_item: String,
}

impl LingeringPotionSplashEvent {
    #[must_use]
    pub const fn new(entity_id: i32, location: BlockPos, potion_item: String) -> Self {
        Self {
            entity_id,
            location,
            potion_item,
            cancelled: false,
        }
    }
}
