use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player responds to a resource pack request.
#[derive(Event, Clone)]
pub struct PlayerResourcePackStatusEvent {
    /// The player who responded.
    pub player: Arc<Player>,

    /// The resource pack UUID.
    pub pack_uuid: uuid::Uuid,

    /// The resource pack hash.
    pub pack_hash: String,

    /// The response status.
    pub status: String,
}

impl PlayerResourcePackStatusEvent {
    /// Creates a new instance of `PlayerResourcePackStatusEvent`.
    pub fn new(player: Arc<Player>, pack_uuid: uuid::Uuid, pack_hash: String, status: String) -> Self {
        Self {
            player,
            pack_uuid,
            pack_hash,
            status,
        }
    }
}

impl PlayerEvent for PlayerResourcePackStatusEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
