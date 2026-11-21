use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// What caused a weather change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherCause {
    /// The weather cycle ran out a timer.
    Natural,
    /// The `/weather` command.
    Command,
    /// A plugin changed the weather.
    Plugin,
    /// Players slept through the night.
    Sleep,
}

/// An event that occurs when weather (raining) changes in a world.
///
/// A natural change caused by a `/weather clear` countdown can not be cancelled.
#[cancellable]
#[derive(Event, Clone)]
pub struct WeatherChangeEvent {
    /// The world where weather is changing.
    pub world: Arc<World>,

    /// The new weather state (true = raining, false = clear).
    pub to_weather_state: bool,

    pub cause: WeatherCause,
}

impl WeatherChangeEvent {
    #[must_use]
    pub const fn new(world: Arc<World>, to_weather_state: bool, cause: WeatherCause) -> Self {
        Self {
            world,
            to_weather_state,
            cause,
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

    pub cause: WeatherCause,
}

impl ThunderChangeEvent {
    #[must_use]
    pub const fn new(world: Arc<World>, to_thunder_state: bool, cause: WeatherCause) -> Self {
        Self {
            world,
            to_thunder_state,
            cause,
            cancelled: false,
        }
    }
}

/// An event that occurs when a custom weather phase of a world starts or ends.
#[cancellable]
#[derive(Event, Clone)]
pub struct WeatherPhaseChangeEvent {
    /// The world where the phase is changing.
    pub world: Arc<World>,

    /// The phase id from the weather profile.
    pub phase: String,

    /// The new state (true = active).
    pub to_active: bool,

    pub cause: WeatherCause,
}

impl WeatherPhaseChangeEvent {
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        phase: String,
        to_active: bool,
        cause: WeatherCause,
    ) -> Self {
        Self {
            world,
            phase,
            to_active,
            cause,
            cancelled: false,
        }
    }
}
