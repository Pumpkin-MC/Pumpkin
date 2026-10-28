#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SContainerSlotStateChanged;

impl JavaClient {
    pub fn handle_container_slot_state_changed(
        &self,
        player: &Player,
        packet: &SContainerSlotStateChanged,
    ) {
        debug!(
            "Player {} container {} slot {} state changed to {}",
            player.gameprofile.name, packet.container_id.0, packet.slot_id.0, packet.new_state
        );
        let _ = player;
    }
}
