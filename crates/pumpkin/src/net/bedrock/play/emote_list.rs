#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_emote_list(&self, player: &Arc<Player>, packet: &SEmoteList) {
        tracing::info!(
            "handle_emote_list: player={} packet={:?}",
            player.gameprofile.name,
            packet
        );
    }
}
