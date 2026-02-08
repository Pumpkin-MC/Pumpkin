use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when thunder state changes in a world.
///
/// If the event is cancelled, the thunder change will not occur.
///
/// Matches Bukkit's `ThunderChangeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct ThunderChangeEvent {
    /// The world where the thunder state is changing.
    pub world: Arc<World>,

    /// Whether it will be thundering after this change.
    pub to_thundering: bool,
}

impl ThunderChangeEvent {
    #[must_use]
    pub fn new(world: Arc<World>, to_thundering: bool) -> Self {
        Self {
            world,
            to_thundering,
            cancelled: false,
        }
    }
}
