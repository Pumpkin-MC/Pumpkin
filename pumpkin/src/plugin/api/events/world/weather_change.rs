use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when the weather changes in a world.
///
/// If the event is cancelled, the weather change will not occur.
///
/// Matches Bukkit's `WeatherChangeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct WeatherChangeEvent {
    /// The world where the weather is changing.
    pub world: Arc<World>,

    /// Whether it will be raining after this change.
    pub to_raining: bool,
}

impl WeatherChangeEvent {
    #[must_use]
    pub const fn new(world: Arc<World>, to_raining: bool) -> Self {
        Self {
            world,
            to_raining,
            cancelled: false,
        }
    }
}
