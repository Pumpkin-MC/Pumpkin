#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_set_test_block(&self, player: &Arc<Player>, packet: &SSetTestBlock<'_>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        debug!(
            "Set test block at {:?}: mode={:?}, message={}",
            packet.position, packet.mode, packet.message
        );
    }
}
