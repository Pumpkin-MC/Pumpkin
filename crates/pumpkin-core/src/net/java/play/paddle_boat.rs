#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_paddle_boat(&self, player: &Arc<Player>, packet: &SPaddleBoat) {
        if let Some(vehicle) = player.get_entity().get_vehicle() {
            vehicle.set_paddle_state(packet.left_paddle, packet.right_paddle);
        }
    }
}
