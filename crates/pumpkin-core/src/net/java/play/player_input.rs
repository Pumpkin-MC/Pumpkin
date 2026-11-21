#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_input(
        &self,
        player: &Arc<Player>,
        input: &SPlayerInput,
        server: &Arc<Server>,
    ) {
        player.apply_client_input(server, input.input);
    }
}
