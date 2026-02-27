use std::sync::Arc;

use pumpkin_macros::Event;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's egg hits something.
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

    /// The entity type that should hatch (registry key).
    pub hatching_type: String,
}

impl PlayerEggThrowEvent {
    /// Creates a new instance of `PlayerEggThrowEvent`.
    pub const fn new(
        player: Arc<Player>,
        egg_uuid: uuid::Uuid,
        hatching: bool,
        num_hatches: u8,
        hatching_type: String,
    ) -> Self {
        Self {
            player,
            egg_uuid,
            hatching,
            num_hatches,
            hatching_type,
        }
    }
}

impl PlayerEvent for PlayerEggThrowEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
