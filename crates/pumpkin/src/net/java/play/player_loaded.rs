#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_loaded(player: &Player) {
        player.set_client_loaded(true);
    }
}
