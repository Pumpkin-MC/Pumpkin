use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player shears an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerShearEntityEvent {
    /// The shearing player.
    pub player: Arc<Player>,

    /// The sheared entity ID.
    pub entity_id: i32,

    /// Hand used (0 = main hand, 1 = off hand).
    pub hand: u8,
}

impl PlayerEvent for PlayerShearEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
