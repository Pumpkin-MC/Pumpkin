#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
        let entity = player.get_entity();
        let pos = Vector3::new(packet.x, packet.y, packet.z);
        let mut final_pos = pos;
        let vehicle = entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(vehicle) = vehicle {
            let vehicle_entity = vehicle.get_entity();
            let from = vehicle_entity.pos.load();
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);

            if let Some(vehicle) = vehicle.get_vehicle_entity() {
                let move_result = vehicle.move_vehicle(from, pos);
                final_pos = move_result.position;
                vehicle_entity.set_pos(final_pos);

                if move_result.cancelled {
                    vehicle_entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    vehicle_entity.send_velocity();
                }

                if final_pos != pos {
                    self.try_send_packet(&CMoveVehicle::new(
                        final_pos.x,
                        final_pos.y,
                        final_pos.z,
                        vehicle_entity.yaw.load(),
                        vehicle_entity.pitch.load(),
                    ));
                }
            }
        }
        entity.set_pos(final_pos);
        chunker::update_position(player);
    }
}
