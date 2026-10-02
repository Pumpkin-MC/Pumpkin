use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

#[derive(Event, Clone)]
pub struct PlayerPermissionCheckEvent {
    pub player: Arc<Player>,
    pub permission: String,
    pub result: bool,
}

impl PlayerPermissionCheckEvent {
    pub const fn new(player: Arc<Player>, permission: String, result: bool) -> Self {
        Self {
            player,
            permission,
            result,
        }
    }
}

impl PlayerEvent for PlayerPermissionCheckEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
