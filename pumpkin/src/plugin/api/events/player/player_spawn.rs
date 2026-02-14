use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

#[derive(Event, Clone)]
pub struct PlayerSpawnEvent {
    pub player: Arc<Player>,
}

impl PlayerSpawnEvent {
    pub const fn new(player: Arc<Player>) -> Self {
        Self { player }
    }
}

impl PlayerEvent for PlayerSpawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
