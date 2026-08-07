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

    /// The previous spawn yaw.
    pub previous_yaw: f32,

    /// The previous spawn pitch.
    pub previous_pitch: f32,

    /// The new spawn position.
    pub new_position: BlockPos,

    /// The new spawn yaw.
    pub new_yaw: f32,

    /// The new spawn pitch.
    pub new_pitch: f32,
}

impl SpawnChangeEvent {
    /// Creates a new `SpawnChangeEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        previous_position: BlockPos,
        previous_yaw: f32,
        previous_pitch: f32,
        new_position: BlockPos,
        new_yaw: f32,
        new_pitch: f32,
    ) -> Self {
        Self {
            world,
            previous_position,
            previous_yaw,
            previous_pitch,
            new_position,
            new_yaw,
            new_pitch,
        }
    }
}
