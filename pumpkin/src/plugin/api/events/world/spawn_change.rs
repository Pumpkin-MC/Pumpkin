use crate::world::World;
use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when the world spawn point changes.
#[derive(Event, Clone)]
pub struct SpawnChangeEvent {
    /// The world whose spawn point changed.
    pub world: Arc<World>,

    /// The previous spawn position.
    pub previous_position: BlockPos,

    /// The new spawn position.
    pub new_position: BlockPos,
}

impl SpawnChangeEvent {
    /// Creates a new `SpawnChangeEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        previous_position: BlockPos,
        new_position: BlockPos,
    ) -> Self {
        Self {
            world,
            previous_position,
            new_position,
        }
    }
}
