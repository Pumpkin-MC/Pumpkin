#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::vehicle::boat::BoatEntity;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
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
        if !player.has_client_loaded()
            || player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
        {
            return;
        }

        let entity = player.get_entity();
        let Some(vehicle) = entity.get_vehicle() else {
            return;
        };
        let vehicle_entity = vehicle.get_entity();
        let is_controller = vehicle_entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .is_some_and(|passenger| passenger.get_entity().entity_id == entity.entity_id);
        if !is_controller {
            return;
        }

        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        let last_pos = vehicle_entity.pos.load();
        let pos = Vector3::new(
            Self::clamp_horizontal(packet.x),
            Self::clamp_vertical(packet.y),
            Self::clamp_horizontal(packet.z),
        );
        if let Some(boat) = vehicle.cast_any().downcast_ref::<BoatEntity>() {
            if !boat.move_from_client(player, pos, packet.yaw, packet.pitch) {
                return;
            }
        } else {
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);
            entity.set_pos(pos);
        }
        let distance = last_pos
            .squared_distance_to_vec(&vehicle_entity.pos.load())
            .sqrt();
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
