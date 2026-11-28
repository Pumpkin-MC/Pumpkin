#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_paddle_boat(&self, player: &Arc<Player>, packet: &SPaddleBoat) {
        if let Some(vehicle) = player.get_entity().get_vehicle()
            && vehicle.get_entity().entity_type.boat.is_some()
            && vehicle.get_controlling_passenger().is_some_and(|driver| {
                driver.get_entity().entity_id == player.get_entity().entity_id
            })
        {
            vehicle.set_paddle_state(packet.left_paddle, packet.right_paddle);
        }
    }
}
