use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when weather (raining) changes in a world.
#[cancellable]
#[derive(Event, Clone)]
pub struct WeatherChangeEvent {
    /// The world where weather is changing.
    pub world: Arc<World>,

    /// The new weather state (true = raining, false = clear).
    pub to_weather_state: bool,
}

impl WeatherChangeEvent {
    #[must_use]
    pub const fn new(world: Arc<World>, to_weather_state: bool) -> Self {
        Self {
            world,
            to_weather_state,
            cancelled: false,
        }
    }
}

/// An event that occurs when thundering state changes in a world.
#[cancellable]
#[derive(Event, Clone)]
pub struct ThunderChangeEvent {
    /// The world where thunder state is changing.
    pub world: Arc<World>,

    /// The new thunder state (true = thundering, false = clear).
    pub to_thunder_state: bool,
}

impl ThunderChangeEvent {
    #[must_use]
    pub const fn new(world: Arc<World>, to_thunder_state: bool) -> Self {
        Self {
            world,
            to_thunder_state,
            cancelled: false,
        }
    }
}
