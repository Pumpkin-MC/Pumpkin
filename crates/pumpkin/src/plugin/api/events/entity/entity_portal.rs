use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity enters a portal to travel between dimensions.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPortalEvent {
    /// The ID of the entity entering the portal.
    pub entity_id: i32,

    /// The position of the portal block.
    pub portal_pos: BlockPos,
}

impl EntityPortalEvent {
    #[must_use]
    pub const fn new(entity_id: i32, portal_pos: BlockPos) -> Self {
        Self {
            entity_id,
            portal_pos,
            cancelled: false,
        }
    }
}
