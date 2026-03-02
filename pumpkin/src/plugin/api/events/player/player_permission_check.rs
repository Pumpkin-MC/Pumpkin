use pumpkin_macros::Event;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

#[derive(Event, Clone)]
pub struct PlayerPermissionCheckEvent {
    pub player: Arc<Player>,
    pub permission: String,
    pub result: bool,
    pub denial_message: Option<TextComponent>,
}

impl PlayerPermissionCheckEvent {
    pub const fn new(player: Arc<Player>, permission: String, result: bool) -> Self {
        Self {
            player,
            permission,
            result,
            denial_message: None,
        }
    }
}

impl PlayerEvent for PlayerPermissionCheckEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
