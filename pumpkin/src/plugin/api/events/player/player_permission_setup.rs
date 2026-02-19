use pumpkin_macros::Event;
use pumpkin_util::permission::PermissionLvl;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

#[derive(Event, Clone)]
pub struct PlayerPermissionSetupEvent {
    pub player: Arc<Player>,
    pub permission_lvl: PermissionLvl,
}

impl PlayerPermissionSetupEvent {
    pub const fn new(player: Arc<Player>, permission_lvl: PermissionLvl) -> Self {
        Self {
            player,
            permission_lvl,
        }
    }
}

impl PlayerEvent for PlayerPermissionSetupEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
