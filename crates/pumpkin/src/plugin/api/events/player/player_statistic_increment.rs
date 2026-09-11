use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player's statistic is incremented.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerStatisticIncrementEvent {
    /// The player whose statistic changed.
    pub player: Arc<Player>,

    /// The statistic identifier name.
    pub statistic_id: String,

    /// The amount incremented.
    pub amount: i32,
}

impl PlayerEvent for PlayerStatisticIncrementEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
