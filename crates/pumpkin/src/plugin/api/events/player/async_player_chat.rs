use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An asynchronous event that occurs when a player sends a chat message.
#[cancellable]
#[derive(Event, Clone)]
pub struct AsyncPlayerChatEvent {
    /// The player sending the message.
    pub player: Arc<Player>,

    /// The chat message content.
    pub message: String,

    /// The formatted chat message.
    pub format: TextComponent,
}

impl PlayerEvent for AsyncPlayerChatEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
