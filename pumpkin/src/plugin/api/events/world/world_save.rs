use pumpkin_macros::Event;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a world is saved.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `WorldSaveEvent`.
#[derive(Event, Clone)]
pub struct WorldSaveEvent {
    /// The world being saved.
    pub world: Arc<World>,
}

impl WorldSaveEvent {
    #[must_use]
    pub const fn new(world: Arc<World>) -> Self {
        Self { world }
    }
}
