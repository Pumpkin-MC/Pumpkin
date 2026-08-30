#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
        let entity = player.get_entity();
        let pos = Vector3::new(packet.x, packet.y, packet.z);
        let vehicle = entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(vehicle) = vehicle {
            let vehicle_entity = vehicle.get_entity();
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);
        }
        entity.set_pos(pos);
        chunker::update_position(player);
    }
}
