use pumpkin_macros::Event;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a world is initialized.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `WorldInitEvent`.
#[derive(Event, Clone)]
pub struct WorldInitEvent {
    /// The world being initialized.
    pub world: Arc<World>,
}

impl WorldInitEvent {
    #[must_use]
    pub const fn new(world: Arc<World>) -> Self {
        Self { world }
    }
}
