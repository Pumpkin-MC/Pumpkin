use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity exits a portal.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPortalExitEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Portal from position.
    pub from_pos: BlockPos,
    /// Destination position.
    pub to_pos: Option<BlockPos>,
}

impl EntityPortalExitEvent {
    #[must_use]
    pub const fn new(entity_id: i32, from_pos: BlockPos, to_pos: Option<BlockPos>) -> Self {
        Self {
            entity_id,
            from_pos,
            to_pos,
            cancelled: false,
        }
    }
}
