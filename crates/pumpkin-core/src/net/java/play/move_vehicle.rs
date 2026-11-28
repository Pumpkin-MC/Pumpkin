use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, fluid::Fluid, tag, translation};
use pumpkin_protocol::java::client::play::CMoveVehicle;
use pumpkin_protocol::java::server::play::SMoveVehicle;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector3::Vector3, wrap_degrees,
};
use pumpkin_util::text::TextComponent;

use crate::entity::player::Player;
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::entity::{EntityBase, position_rider};
use crate::net::java::JavaClient;
use crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent;
use crate::server::Server;
use crate::world::chunker;

#[derive(Clone, Copy)]
pub(in crate::net::java) struct VehicleMovement {
    root_id: i32,
    first_good: Vector3<f64>,
    last_good: Vector3<f64>,
    last_rider_position: Option<(u64, Vector3<f64>)>,
    floating: bool,
    floating_ticks: u16,
}

fn root_vehicle(player: &Player) -> Option<Arc<dyn EntityBase>> {
    player.get_entity().get_root_vehicle().filter(|root| {
        let entity = root.get_entity();
        !entity.is_removed() && Arc::ptr_eq(&entity.world.load(), &player.world())
    })
}

fn controls(player: &Player, root: &dyn EntityBase) -> bool {
    root.get_controlling_passenger()
        .is_some_and(|rider| rider.get_entity().entity_id == player.get_entity().entity_id)
}

fn correct_vehicle(client: &JavaClient, player: &Player, vehicle: &dyn EntityBase) {
    let entity = vehicle.get_entity();
    let pos = entity.pos.load();
    let yaw = entity.yaw.load();
    let pitch = entity.pitch.load();
    let rider = player.get_entity();
    let revision = rider.riding_revision();
    let teleport = player.teleport_id_count.load(Ordering::Acquire);
    let Ok(data) = client.serialize_packet(&CMoveVehicle::new(pos.x, pos.y, pos.z, yaw, pitch))
    else {
        return;
    };
    // The packet has no entity ID. The client applies it to its current root.
    client.try_enqueue_packet_data_if(
        data,
        CMoveVehicle::to_id(client.version.load()),
        |_| true,
        || {
            let root = root_vehicle(player)?;
            if !std::ptr::eq(root.get_entity(), entity) {
                return None;
            }
            let awaiting = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (awaiting.is_none()
                && player.teleport_id_count.load(Ordering::Acquire) == teleport
                && rider.riding_revision() == revision
                && entity.pos.load() == pos
                && entity.yaw.load().to_bits() == yaw.to_bits()
                && entity.pitch.load().to_bits() == pitch.to_bits())
            .then_some(awaiting)
        },
    );
}

impl JavaClient {
    /// Capture the authority and position baseline before this world's packet queues are drained.
    pub(crate) fn prepare_vehicle_movement(&self, player: &Player) -> bool {
        let state = root_vehicle(player)
            .filter(|root| controls(player, root.as_ref()))
            .map(|root| {
                let entity = root.get_entity();
                let pos = entity.pos.load();
                let previous = self
                    .vehicle_movement
                    .load()
                    .filter(|state| state.root_id == entity.entity_id);
                let floating = previous.is_some_and(|state| state.floating)
                    && player.world().server.upgrade().is_some_and(|server| {
                        !server
                            .management_hub
                            .settings
                            .allow_flight
                            .load(Ordering::Relaxed)
                    });
                let floating_ticks = previous
                    .filter(|_| floating)
                    .map_or(0, |state| state.floating_ticks.saturating_add(1));
                if floating_ticks > 160 {
                    self.try_kick(&TextComponent::translate(
                        translation::java::MULTIPLAYER_DISCONNECT_FLYING,
                        [],
                    ));
                }
                VehicleMovement {
                    root_id: entity.entity_id,
                    first_good: pos,
                    last_good: pos,
                    last_rider_position: None,
                    floating,
                    floating_ticks,
                }
            });
        self.vehicle_movement.store(state);
        state.is_some()
    }

    pub(crate) fn last_vehicle_rider_position(&self, attachment: u64) -> Option<Vector3<f64>> {
        let (revision, position) = self.vehicle_movement.load()?.last_rider_position?;
        (revision == attachment).then_some(position)
    }

    pub fn handle_move_vehicle(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SMoveVehicle,
    ) {
        if packet.x.is_nan()
            || packet.y.is_nan()
            || packet.z.is_nan()
            || !packet.yaw.is_finite()
            || !packet.pitch.is_finite()
        {
            self.try_kick(&TextComponent::translate(
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
        let Some(root) = root_vehicle(player) else {
            return;
        };
        let vehicle = root.get_entity();
        if !controls(player, root.as_ref()) {
            let tick = player.tick_counter.load(Ordering::Relaxed);
            let sent_at = self.vehicle_correction_tick.load(Ordering::Relaxed);
            let requested = Vector3::new(packet.x, packet.y, packet.z);
            if tick.wrapping_sub(sent_at) > 20
                || (vehicle.pos.load() - requested).length_squared() > 1.0
            {
                self.vehicle_correction_tick.store(tick, Ordering::Relaxed);
                correct_vehicle(self, player, root.as_ref());
            }
            return;
        }
        let Some(mut state) = self
            .vehicle_movement
            .load()
            .filter(|state| state.root_id == vehicle.entity_id)
        else {
            return;
        };
        let vehicle_pos = vehicle.pos.load();
        if vehicle_pos != state.last_good {
            state.first_good = vehicle_pos;
            state.last_good = vehicle_pos;
            state.floating = false;
            state.floating_ticks = 0;
            self.vehicle_movement.store(Some(state));
        }
        let proposed = bounded_position(Vector3::new(packet.x, packet.y, packet.z));
        let too_fast = |target: Vector3<f64>| {
            (target - state.first_good).length_squared() - vehicle.velocity.load().length_squared()
                > 100.0
        };
        if player.is_movement_locked() {
            // The stock client echoes its prediction unchanged inside this tolerance.
            const CLIENT_SNAP_DISTANCE: f32 = 1.0e-5;
            if (proposed - vehicle.pos.load()).length() > f64::from(CLIENT_SNAP_DISTANCE) {
                correct_vehicle(self, player, root.as_ref());
            }
            return;
        }
        if too_fast(proposed) {
            correct_vehicle(self, player, root.as_ref());
            return;
        }

        let old_pos = vehicle.pos.load();
        let Some(target) =
            vehicle_move_destination(player, server, root.as_ref(), old_pos, proposed)
                .filter(|&target| !too_fast(target))
        else {
            if !vehicle.is_removed() && controls(player, root.as_ref()) {
                correct_vehicle(self, player, root.as_ref());
            }
            return;
        };

        self.apply_vehicle_move(player, server, root.as_ref(), state, target, packet);
    }

    fn apply_vehicle_move(
        &self,
        player: &Arc<Player>,
        server: &Server,
        root: &dyn EntityBase,
        mut state: VehicleMovement,
        target: Vector3<f64>,
        packet: &SMoveVehicle,
    ) {
        let vehicle = root.get_entity();
        let old_pos = vehicle.pos.load();
        let requested = target - state.last_good;
        let motion = vehicle.apply_stuck_speed_multiplier(requested);
        let collision = vehicle.project_movement(motion, root);
        let bounds = vehicle.bounding_box.load();
        let world = vehicle.world.load_full();
        let epoch = vehicle.vehicle_motion_epoch();
        let records_contact = root.vehicle_contact_tick_reach().is_some();
        let no_physics = vehicle.no_physics.load(Ordering::Relaxed);

        let was_supported = vehicle.vertical_collision_below.load(Ordering::Relaxed);
        if let Some(living) = root.get_living_entity()
            && (living.on_climbable()
                || (living.fall_distance.load() > 0.0
                    && clips_fall_resetting_block(&world, old_pos, collision.movement)))
        {
            living.fall_distance.store(0.0);
        }
        vehicle.apply_client_vehicle_collision(motion, &collision);
        if no_physics
            || !records_contact
            || vehicle.record_vehicle_movement(old_pos, collision.movement, motion)
        {
            vehicle.set_pos(old_pos + collision.movement);
        }
        let resolved = vehicle.pos.load();
        if !no_physics {
            let attachment = player.get_entity().riding_revision();
            vehicle.emit_client_movement(root, collision.movement);
            if vehicle.is_removed()
                || vehicle.pos.load() != resolved
                || vehicle.vehicle_motion_epoch() != epoch
                || !Arc::ptr_eq(&world, &vehicle.world.load())
                || player.get_entity().riding_revision() != attachment
                || !controls(player, root)
            {
                return;
            }
            let factor = f64::from(root.get_block_speed_factor());
            let velocity = vehicle.velocity.load();
            vehicle
                .velocity
                .store(velocity.multiply(factor, 1.0, factor));
        }
        // Java's vertical-discrepancy condition makes that term zero for every real value.
        let moved_wrongly = (target - resolved).horizontal_length_squared() > 0.0625;
        let rejected = (moved_wrongly && world.no_vehicle_collision(root, bounds))
            || world.has_new_vehicle_collision(root, bounds, target - old_pos);
        vehicle.set_rotation(wrap_degrees(packet.yaw), wrap_degrees(packet.pitch));
        if rejected {
            vehicle.set_pos(old_pos);
            if records_contact {
                vehicle.discard_last_vehicle_movement();
            }
            correct_vehicle(self, player, root);
            return;
        }

        let delta = target - old_pos;
        vehicle.set_pos(target);
        vehicle.set_vehicle_packet_ground(root, packet.on_ground, delta);
        if let Some(boat) = root
            .cast_any()
            .downcast_ref::<crate::entity::vehicle::boat::BoatEntity>()
        {
            boat.update_last_vertical_speed();
        }
        if let Some(living) = root.get_living_entity() {
            living.fall(root, delta.y, packet.on_ground);
        }
        if player
            .get_entity()
            .get_vehicle()
            .is_some_and(|mount| mount.get_entity().entity_id == vehicle.entity_id)
        {
            position_rider(root, player.as_ref());
            let rider = player.get_entity();
            state.last_rider_position = Some((rider.riding_revision(), rider.pos.load()));
        }
        state.last_good = target;
        state.floating = root
            .client_vehicle_float_threshold()
            .is_some_and(|minimum| requested.y >= minimum)
            && !was_supported
            && !vehicle.has_no_gravity()
            && !server
                .management_hub
                .settings
                .allow_flight
                .load(Ordering::Relaxed)
            && no_blocks_around(&world, vehicle.bounding_box.load());
        self.vehicle_movement.store(Some(state));
        self.commit_vehicle_rider_move(player, root, target, delta);
    }

    fn commit_vehicle_rider_move(
        &self,
        player: &Arc<Player>,
        root: &dyn EntityBase,
        target: Vector3<f64>,
        delta: Vector3<f64>,
    ) {
        if delta.length_squared() > f64::from(1.0e-5f32) {
            player.update_last_action_time();
        }
        player.get_entity().movement.store(delta);
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        let cm = (delta.length() * 100.0).round() as i32;
        if cm > 0 && !root.get_entity().uses_vehicle_collisions() {
            player.increment_custom_stat(player.get_movement_statistic(), cm);
        }
        check_ride_in_lava(player, root, target);
        chunker::update_position(player);
    }
}

fn vehicle_move_destination(
    player: &Player,
    server: &Arc<Server>,
    root: &dyn EntityBase,
    from: Vector3<f64>,
    to: Vector3<f64>,
) -> Option<Vector3<f64>> {
    if from == to || !server.plugin_manager.has_handlers::<VehicleMoveEvent>() {
        return Some(to);
    }
    let rider = player.get_entity();
    let vehicle = root.get_entity();
    let world = vehicle.world.load_full();
    let epoch = vehicle.vehicle_motion_epoch();
    let revision = rider.riding_revision();
    let teleport_id = player.teleport_id_count.load(Ordering::Acquire);
    let event = VehicleMoveEvent::new(vehicle.entity_id, from, to);
    let event = server.plugin_manager.fire_blocking(server, event);
    let valid = !event.cancelled
        && !player.is_movement_locked()
        && event.to.x.is_finite()
        && event.to.y.is_finite()
        && event.to.z.is_finite()
        && rider.riding_revision() == revision
        && player.teleport_id_count.load(Ordering::Acquire) == teleport_id
        && !vehicle.is_removed()
        && vehicle.pos.load() == from
        && vehicle.vehicle_motion_epoch() == epoch
        && Arc::ptr_eq(&world, &player.world())
        && Arc::ptr_eq(&world, &vehicle.world.load())
        && controls(player, root);
    valid.then(|| bounded_position(event.to))
}

const fn bounded_position(position: Vector3<f64>) -> Vector3<f64> {
    Vector3::new(
        position.x.clamp(-3.0e7, 3.0e7),
        position.y.clamp(-2.0e7, 2.0e7),
        position.z.clamp(-3.0e7, 3.0e7),
    )
}

fn check_ride_in_lava(player: &Player, vehicle: &dyn EntityBase, pos: Vector3<f64>) {
    let world = player.world();
    let vehicle_pos = pos.to_block_pos();
    let below = BlockPos::new(vehicle_pos.0.x, vehicle_pos.0.y - 1, vehicle_pos.0.z);
    if world.get_block(&vehicle_pos) != &Block::LAVA && world.get_block(&below) != &Block::LAVA {
        player.entered_lava_pos.store(None);
        return;
    }
    let start = player.entered_lava_pos.load().unwrap_or(pos);
    player.entered_lava_pos.store(Some(start));
    player.trigger_advancement(AdvancementTrigger::RideEntityInLava {
        distance_horizontal: (pos - start).horizontal_length(),
        dimension: world.dimension.minecraft_name.to_string(),
        vehicle_type: Some(format!(
            "minecraft:{}",
            vehicle.get_entity().entity_type.resource_name
        )),
    });
}

fn no_blocks_around(world: &crate::world::World, bounds: BoundingBox) -> bool {
    let area = bounds.expand_all(0.0625).expand_towards(0.0, -0.55, 0.0);
    BlockPos::iterate(area.min_block_pos(), area.max_block_pos())
        .all(|pos| world.get_block_state(&pos).is_air())
}

fn clips_fall_resetting_block(
    world: &crate::world::World,
    from: Vector3<f64>,
    movement: Vector3<f64>,
) -> bool {
    let length = movement.length();
    if length < 1.0 {
        return false;
    }
    let ray = movement * (length.min(8.0) / length);
    let to = from + ray;
    let area = BoundingBox::new(
        Vector3::new(from.x.min(to.x), from.y.min(to.y), from.z.min(to.z)),
        Vector3::new(from.x.max(to.x), from.y.max(to.y), from.z.max(to.z)),
    );
    BlockPos::iterate(area.min_block_pos(), area.max_block_pos()).any(|pos| {
        let block_box = BoundingBox::from_block(&pos);
        if world
            .get_block(&pos)
            .has_tag(&tag::Block::MINECRAFT_FALL_DAMAGE_RESETTING)
            && block_box.ray_intersection(from, ray).is_some()
        {
            return true;
        }
        let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
        if !fluid.matches_type(&Fluid::WATER) {
            return false;
        }
        let max = Vector3::new(
            block_box.max.x,
            block_box.min.y + f64::from(world.get_fluid_height(&pos, fluid, &state)),
            block_box.max.z,
        );
        BoundingBox::new(block_box.min, max)
            .ray_intersection(from, ray)
            .is_some()
    })
}
