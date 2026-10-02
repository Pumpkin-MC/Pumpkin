use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity enters a portal.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPortalEnterEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Location of the portal block entered.
    pub location: BlockPos,
}

impl EntityPortalEnterEvent {
    #[must_use]
    pub const fn new(entity_id: i32, location: BlockPos) -> Self {
        Self {
            entity_id,
            location,
            cancelled: false,
        }
    }
}
