#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_close_container(&self, player: &Arc<Player>) {
        player.on_handled_screen_closed();
    }
}
