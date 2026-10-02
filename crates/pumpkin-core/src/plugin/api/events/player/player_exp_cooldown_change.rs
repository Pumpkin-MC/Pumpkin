use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player's experience cooldown changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerExpCooldownChangeEvent {
    /// The player involved.
    pub player: Arc<Player>,

    /// The new cooldown ticks.
    pub new_cooldown: i32,
}

impl PlayerEvent for PlayerExpCooldownChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
