use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;
use crate::world::World;

/// An event that occurs after a player has changed worlds.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerChangedWorldEvent {
    /// The player who changed worlds.
    pub player: Arc<Player>,

    /// The world the player came from.
    pub from_world: Arc<World>,

    /// The world the player arrived in.
    pub to_world: Arc<World>,
}

impl PlayerEvent for PlayerChangedWorldEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
