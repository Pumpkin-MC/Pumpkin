use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player sends a custom payload packet.
#[derive(Event, Clone)]
pub struct PlayerCustomPayloadEvent {
    /// The player who sent the custom payload.
    pub player: Arc<Player>,
    /// The payload channel identifier (e.g. `voicechat:request_secret`).
    pub channel: String,
    /// The raw payload data.
    pub data: Vec<u8>,
}

impl PlayerCustomPayloadEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, channel: String, data: Vec<u8>) -> Self {
        Self {
            player,
            channel,
            data,
        }
    }
}

impl PlayerEvent for PlayerCustomPayloadEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
