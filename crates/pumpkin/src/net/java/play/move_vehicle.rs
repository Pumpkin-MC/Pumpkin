#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_move_vehicle(&self, player: &Arc<Player>, packet: SMoveVehicle) {
        let entity = player.get_entity();
        let pos = Vector3::new(packet.x, packet.y, packet.z);
        let vehicle = entity.vehicle.lock().await;
        if let Some(vehicle) = vehicle.as_ref() {
            let vehicle_entity = vehicle.get_entity();
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);
        }
        drop(vehicle);
        entity.set_pos(pos);
        chunker::update_position(player).await;
    }
}
