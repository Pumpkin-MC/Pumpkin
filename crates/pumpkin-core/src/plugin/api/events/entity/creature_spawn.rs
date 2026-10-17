use crate::entity::player::Player;
use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

/// An event that occurs when a creature spawns.
#[cancellable]
#[derive(Event, Clone)]
pub struct CreatureSpawnEvent {
    /// The ID of the spawned creature.
    pub entity_id: i32,

    /// The registry name of the entity type.
    pub entity_type: String,

    /// The position where the creature spawned.
    pub position: Vector3<f64>,

    /// The world in which the creature spawned.
    pub world: Arc<World>,

    /// The reason why the creature spawned, see [`CreatureSpawnReason`].
    pub spawn_reason: String,

    /// The player who caused the spawn, `None` for other sources such as a dispenser.
    pub player: Option<Arc<Player>>,
}

/// Why a creature spawned, sent to plugins as [`CreatureSpawnReason::as_str`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CreatureSpawnReason {
    /// A player used a spawn egg, including making a baby from a mob.
    SpawnerEgg,
    /// A dispenser dispensed a spawn egg.
    DispenseEgg,
}

impl CreatureSpawnReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SpawnerEgg => "spawner_egg",
            Self::DispenseEgg => "dispense_egg",
        }
    }
}

impl CreatureSpawnEvent {
    #[must_use]
    pub fn new(
        entity_id: i32,
        entity_type: String,
        position: Vector3<f64>,
        world: Arc<World>,
        spawn_reason: CreatureSpawnReason,
        player: Option<Arc<Player>>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            position,
            world,
            spawn_reason: spawn_reason.as_str().to_string(),
            player,
            cancelled: false,
        }
    }
}
