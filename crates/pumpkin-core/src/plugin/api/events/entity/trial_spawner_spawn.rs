use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a trial spawner spawns an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct TrialSpawnerSpawnEvent {
    /// Spawned entity ID.
    pub entity_id: i32,
    /// Trial spawner block position.
    pub spawner_pos: BlockPos,
}

impl TrialSpawnerSpawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32, spawner_pos: BlockPos) -> Self {
        Self {
            entity_id,
            spawner_pos,
            cancelled: false,
        }
    }
}
