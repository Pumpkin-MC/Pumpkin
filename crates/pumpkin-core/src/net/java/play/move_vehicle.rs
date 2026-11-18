#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::vehicle::boat::BoatEntity;
use pumpkin_util::math::wrap_degrees;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
        if !(packet.x.is_finite()
            && packet.y.is_finite()
            && packet.z.is_finite()
            && packet.yaw.is_finite()
            && packet.pitch.is_finite())
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
            player.entered_lava_pos.store(None);
            return;
        };
        let vehicle_entity = vehicle.get_entity();
        let boat = vehicle.cast_any().downcast_ref::<BoatEntity>();
        let controller = vehicle.get_controlling_passenger().or_else(|| {
            if boat.is_some() {
                return None;
            }
            vehicle_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .first()
                .cloned()
        });
        if controller.is_none_or(|rider| rider.get_entity().entity_id != entity.entity_id) {
            return;
        }

        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        let requested = Vector3::new(
            Player::clamp_horizontal(packet.x),
            Player::clamp_vertical(packet.y),
            Player::clamp_horizontal(packet.z),
        );
        let yaw = wrap_degrees(packet.yaw);
        let pitch = wrap_degrees(packet.pitch);
        let (old_pos, pos) = if let Some(boat) = boat {
            let Some(accepted) =
                boat.accept_client_movement(player, requested, yaw, pitch, packet.on_ground)
            else {
                if let Some(correction) = boat.client_movement_correction(player) {
                    player.try_send_client_packet(&correction);
                }
                return;
            };
            accepted
        } else {
            let old_pos = vehicle_entity.pos.load();
            vehicle_entity.set_pos(requested);
            vehicle_entity.set_rotation(yaw, pitch);
            vehicle_entity
                .on_ground
                .store(packet.on_ground, Ordering::Relaxed);
            (old_pos, requested)
        };
        let movement = pos - old_pos;
        if boat.is_none() {
            vehicle_entity.emit_client_movement(vehicle.as_ref(), movement);
        }
        if let Some(living) = vehicle.get_living_entity() {
            living.fall(vehicle.as_ref(), movement.y, packet.on_ground);
        }
        track_vehicle_in_lava(player, vehicle_entity, pos);
        crate::entity::position_rider(vehicle.as_ref(), player.as_ref());
        let cm = (movement.length() * 100.0).round() as i32;
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

fn track_vehicle_in_lava(player: &Player, vehicle: &crate::entity::Entity, pos: Vector3<f64>) {
    let world = player.world();
    let vehicle_pos = pos.to_block_pos();
    let in_lava = world.get_block(&vehicle_pos) == &Block::LAVA
        || world.get_block(&vehicle_pos.down()) == &Block::LAVA;
    if !in_lava {
        player.entered_lava_pos.store(None);
        return;
    }
    let start_pos = player.entered_lava_pos.load().unwrap_or_else(|| {
        player.entered_lava_pos.store(Some(pos));
        pos
    });
    player.trigger_advancement(
        crate::entity::player::advancement::trigger::AdvancementTrigger::RideEntityInLava {
            distance_horizontal: (pos.x - start_pos.x).hypot(pos.z - start_pos.z),
            dimension: world.dimension.minecraft_name.to_string(),
            vehicle_type: Some(format!("minecraft:{}", vehicle.entity_type.resource_name)),
        },
    );
}
