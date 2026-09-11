use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player activates the riptide trident enchantment.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRiptideEvent {
    /// The player activating riptide.
    pub player: Arc<Player>,

    /// The riptide item name.
    pub item_name: String,
}

impl PlayerEvent for PlayerRiptideEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
