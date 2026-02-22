use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player statistic is incremented.
#[derive(Event, Clone)]
pub struct PlayerStatisticIncrementEvent {
    /// The player whose statistic increased.
    pub player: Arc<Player>,

    /// The statistic name.
    pub statistic: String,

    /// The previous value.
    pub initial_value: i32,

    /// The new value.
    pub new_value: i32,

    /// Optional entity type.
    pub entity_type: String,

    /// Optional material key.
    pub material_key: String,
}

impl PlayerStatisticIncrementEvent {
    /// Creates a new instance of `PlayerStatisticIncrementEvent`.
    pub const fn new(
        player: Arc<Player>,
        statistic: String,
        initial_value: i32,
        new_value: i32,
        entity_type: String,
        material_key: String,
    ) -> Self {
        Self {
            player,
            statistic,
            initial_value,
            new_value,
            entity_type,
            material_key,
        }
    }
}

impl PlayerEvent for PlayerStatisticIncrementEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
