use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a world is loaded.
#[derive(Event, Clone)]
pub struct WorldLoadEvent {
    /// The world that was loaded.
    pub world: Arc<World>,
}

impl WorldLoadEvent {
    #[must_use]
    pub const fn new(world: Arc<World>) -> Self {
        Self { world }
    }
}

/// An event that occurs when a world is unloaded.
#[cancellable]
#[derive(Event, Clone)]
pub struct WorldUnloadEvent {
    /// The world being unloaded.
    pub world: Arc<World>,
}

impl WorldUnloadEvent {
    #[must_use]
    pub const fn new(world: Arc<World>) -> Self {
        Self {
            world,
            cancelled: false,
        }
    }
}
