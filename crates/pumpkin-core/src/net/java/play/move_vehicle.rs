use pumpkin_protocol::java::client::play::CMoveVehicle;

#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);

        if !packet.x.is_finite()
            || !packet.y.is_finite()
            || !packet.z.is_finite()
            || !packet.yaw.is_finite()
            || !packet.pitch.is_finite()
        {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_VEHICLE_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_VEHICLE_MOVEMENT,
                [],
            ));
            return;
        }

        let entity = player.get_entity();
        let vehicle = entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        // With no vehicle there is nothing for this packet to move, and applying it
        // to the player instead would skip every check `handle_player_position`
        // makes on the same coordinates.
        let Some(vehicle) = vehicle else {
            return;
        };

        let vehicle_entity = vehicle.get_entity();
        // Only the controlling passenger (the first one) steers.
        let is_controlling = vehicle_entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .is_some_and(|passenger| passenger.get_entity().entity_id == entity.entity_id);
        if !is_controlling {
            return;
        }

        let last_pos = entity.pos.load();
        let pos = Vector3::new(
            crate::net::clamp_horizontal_position(packet.x),
            crate::net::clamp_vertical_position(packet.y),
            crate::net::clamp_horizontal_position(packet.z),
        );

        // Vanilla's "moved too quickly" check, net of the vehicle's own motion. The
        // vehicle is sent back so the client doesn't stay out of sync.
        let vehicle_pos = vehicle_entity.pos.load();
        let expected = vehicle_entity.velocity.load().length_squared();
        if vehicle_pos.squared_distance_to_vec(&pos) - expected > 100.0 {
            warn!(
                "{} moved a vehicle too quickly ({vehicle_pos:?} -> {pos:?})",
                player.gameprofile.name
            );
            self.try_send_packet(&CMoveVehicle::new(
                vehicle_pos.x,
                vehicle_pos.y,
                vehicle_pos.z,
                vehicle_entity.yaw.load(),
                vehicle_entity.pitch.load(),
            ));
            return;
        }

        vehicle_entity.set_pos(pos);
        vehicle_entity.set_rotation(packet.yaw, packet.pitch);
        entity.set_pos(pos);

        let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
        let cm = (distance * 100.0).round() as i32;
        if cm > 0 {
            let stat = player.get_movement_statistic();
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                stat as i32,
                cm,
            );
        }
        chunker::update_position(player);
    }
}
