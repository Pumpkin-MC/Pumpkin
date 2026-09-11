use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a splash potion impacts and applies effects.
#[cancellable]
#[derive(Event, Clone)]
pub struct PotionSplashEvent {
    /// Potion entity ID.
    pub entity_id: i32,
    /// Splash location.
    pub location: BlockPos,
    /// Potion item name.
    pub potion_item: String,
    /// Affected entity IDs.
    pub affected_entities: Vec<i32>,
}

impl PotionSplashEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        location: BlockPos,
        potion_item: String,
        affected_entities: Vec<i32>,
    ) -> Self {
        Self {
            entity_id,
            location,
            potion_item,
            affected_entities,
            cancelled: false,
        }
    }
}
