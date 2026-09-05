#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_set_local_player_as_initialized(
        &self,
        player: &Arc<Player>,
        packet: &SSetLocalPlayerAsInitialized,
    ) {
        debug!(
            "Player {} initialized (Runtime ID: {})",
            player.gameprofile.name, packet.player_id.0
        );
        // This is sent when the client has finished loading and rendering the world.
        player.set_client_loaded(true);
    }
}
