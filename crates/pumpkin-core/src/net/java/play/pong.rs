#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SPlayPong;

impl JavaClient {
    pub fn handle_play_pong(&self, player: &Player, packet: &SPlayPong) {
        debug!(
            "Received pong from player {} with id {}",
            player.gameprofile.name, packet.id
        );
    }
}
