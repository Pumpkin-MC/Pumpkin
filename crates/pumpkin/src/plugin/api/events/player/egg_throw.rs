use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's egg hits something.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerEggThrowEvent {
    /// The player who threw the egg.
    pub player: Arc<Player>,

    /// The UUID of the egg entity.
    pub egg_uuid: uuid::Uuid,

    /// Whether the egg should hatch.
    pub hatching: bool,

    /// The number of entities that should hatch.
    pub num_hatches: u8,

    /// The entity type that should hatch.
    pub hatching_type: &'static EntityType,
}

impl PlayerEggThrowEvent {
    /// Creates a new instance of `PlayerEggThrowEvent`.
    pub const fn new(
        player: Arc<Player>,
        egg_uuid: uuid::Uuid,
        hatching: bool,
        num_hatches: u8,
        hatching_type: &'static EntityType,
    ) -> Self {
        Self {
            player,
            egg_uuid,
            hatching,
            num_hatches,
            hatching_type,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerEggThrowEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
