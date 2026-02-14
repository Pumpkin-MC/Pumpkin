

use std::sync::Arc;
use pumpkin_macros::Event;
use crate::entity::player::Player;
use crate::plugin::player::player_join::PlayerJoinEvent;
use super::PlayerEvent;

#[derive(Event, Clone)]
pub struct PlayerSpawnEvent {
    pub player: Arc<Player>
}

impl PlayerSpawnEvent {

    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player
        }
    }

}

impl PlayerEvent for PlayerSpawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}