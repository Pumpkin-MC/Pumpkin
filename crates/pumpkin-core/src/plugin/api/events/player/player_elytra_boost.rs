use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player boosts their elytra flight using a firework.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerElytraBoostEvent {
    /// The gliding player.
    pub player: Arc<Player>,

    /// The ID of the firework entity.
    pub firework_id: i32,
}

impl PlayerEvent for PlayerElytraBoostEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
