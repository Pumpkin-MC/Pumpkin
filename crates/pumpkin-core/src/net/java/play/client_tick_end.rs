#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_client_tick_end(&self, player: &Player) {
        // If no movement packet arrived this tick the player is standing still —
        // zero out the known movement so consumers (e.g. riptide speed) see 0.
        // Matches vanilla ServerGamePacketListenerImpl#handleClientTickEnd.
        if !self
            .received_movement_this_tick
            .swap(false, Ordering::Relaxed)
        {
            player
                .get_entity()
                .movement
                .store(Vector3::new(0.0, 0.0, 0.0));
        }
    }
}
