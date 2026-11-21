use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// A request to transfer a player to another server.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerTransferEvent {
    pub player: Arc<Player>,
    pub host: String,
    pub port: u16,
}

impl PlayerTransferEvent {
    /// Creates a transfer request that handlers can cancel or redirect.
    pub const fn new(player: Arc<Player>, host: String, port: u16) -> Self {
        Self {
            player,
            host,
            port,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerTransferEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
