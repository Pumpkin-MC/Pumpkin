use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player completes an advancement.
#[derive(Event, Clone)]
pub struct PlayerAdvancementDoneEvent {
    /// The player who completed the advancement.
    pub player: Arc<Player>,

    /// Namespaced advancement key, e.g. "`minecraft:story/mine_diamond`".
    pub advancement_key: String,
}

impl PlayerAdvancementDoneEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, advancement_key: String) -> Self {
        Self {
            player,
            advancement_key,
        }
    }
}

impl PlayerEvent for PlayerAdvancementDoneEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
