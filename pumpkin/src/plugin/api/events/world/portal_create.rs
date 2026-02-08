use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a portal is created.
///
/// If the event is cancelled, the portal will not be created.
///
/// Matches Bukkit's `PortalCreateEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PortalCreateEvent {
    /// The world where the portal is being created.
    pub world: Arc<World>,

    /// The position of the portal.
    pub position: BlockPos,

    /// The reason for the portal creation (e.g. "fire", "end_platform").
    pub reason: String,
}

impl PortalCreateEvent {
    #[must_use]
    pub fn new(world: Arc<World>, position: BlockPos, reason: String) -> Self {
        Self {
            world,
            position,
            reason,
            cancelled: false,
        }
    }
}
