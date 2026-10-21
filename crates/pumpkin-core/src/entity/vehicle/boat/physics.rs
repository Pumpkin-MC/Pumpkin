use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use super::BoatEntity;
use crate::entity::{EntityBase, player::Player};
use crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent;
use pumpkin_protocol::java::client::play::CMoveVehicle;

pub(super) const DEFAULT_GRAVITY: f64 = 0.04;
const WATER_OFFSET: f64 = 0.001;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BoatStatus {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

impl BoatStatus {
    pub(super) const fn is_underwater(self) -> bool {
        matches!(self, Self::UnderWater | Self::UnderFlowingWater)
    }
}

impl BoatEntity {
    pub(in crate::entity) fn modify_passenger_fluid_interaction_box(
        &self,
        mut bounds: BoundingBox,
    ) -> Option<BoundingBox> {
        if self.status.load().is_some_and(BoatStatus::is_underwater) {
            return Some(bounds);
        }
        let top = self.vehicle.entity.bounding_box.load().max.y;
        if top >= bounds.max.y {
            return None;
        }
        bounds.min.y = bounds.min.y.max(top);
        Some(bounds)
    }

    pub(super) fn is_eye_in_water(&self) -> bool {
        let entity = &self.vehicle.entity;
        let eye = entity.get_eye_pos();
        let bounds = entity.bounding_box.load();
        let min_y = (bounds.min.y + WATER_OFFSET).floor() as i32;
        let max_y = (bounds.max.y - WATER_OFFSET).ceil() as i32;
        let world = entity.world.load();
        for y in min_y..max_y {
            let pos = BlockPos::new(eye.x.floor() as i32, y, eye.z.floor() as i32);
            let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
            if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) || eye.y < f64::from(y) {
                continue;
            }
            let height = if state.is_source
                && world
                    .get_block_state(&pos.up())
                    .is_side_solid(BlockDirection::Down)
            {
                1.0
            } else {
                world.get_fluid_height(&pos, fluid, &state)
            };
            if eye.y <= f64::from(y) + f64::from(height) {
                return true;
            }
        }
        false
    }

    pub(super) fn get_status(&self) -> BoatStatus {
        if let Some(status) = self.is_underwater() {
            self.water_level
                .store(self.vehicle.entity.bounding_box.load().max.y);
            return status;
        }
        if self.check_in_water() {
            return BoatStatus::InWater;
        }
        let friction = self.get_ground_friction();
        if friction > 0.0 {
            self.land_friction.store(friction);
            BoatStatus::OnLand
        } else {
            BoatStatus::InAir
        }
    }

    fn is_underwater(&self) -> Option<BoatStatus> {
        let bounds = self.vehicle.entity.bounding_box.load();
        let surface = bounds.max.y + WATER_OFFSET;
        let min = BlockPos::floored(bounds.min.x, bounds.max.y, bounds.min.z);
        let max = BlockPos::new(
            bounds.max.x.ceil() as i32 - 1,
            surface.ceil() as i32 - 1,
            bounds.max.z.ceil() as i32 - 1,
        );
        let world = self.vehicle.entity.world.load();
        let mut underwater = false;
        for pos in BlockPos::iterate(min, max) {
            let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
            if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)
                || surface
                    >= f64::from(pos.0.y as f32 + world.get_fluid_height(&pos, fluid, &state))
            {
                continue;
            }
            if !state.is_source {
                return Some(BoatStatus::UnderFlowingWater);
            }
            underwater = true;
        }
        underwater.then_some(BoatStatus::UnderWater)
    }

    fn check_in_water(&self) -> bool {
        let bounds = self.vehicle.entity.bounding_box.load();
        let min = bounds.min_block_pos();
        let max = BlockPos::new(
            bounds.max.x.ceil() as i32 - 1,
            (bounds.min.y + WATER_OFFSET).ceil() as i32 - 1,
            bounds.max.z.ceil() as i32 - 1,
        );
        let world = self.vehicle.entity.world.load();
        let mut water_level = -f64::MAX;
        let mut in_water = false;
        for pos in BlockPos::iterate(min, max) {
            let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
            if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                continue;
            }
            // Vanilla adds the block Y as a float before widening to double.
            let height = f64::from(pos.0.y as f32 + world.get_fluid_height(&pos, fluid, &state));
            water_level = water_level.max(height);
            in_water |= bounds.min.y < height;
        }
        self.water_level.store(water_level);
        in_water
    }

    fn get_water_level_above(&self) -> f32 {
        let bounds = self.vehicle.entity.bounding_box.load();
        let top = (bounds.max.y - self.last_yd.load()).ceil() as i32;
        let world = self.vehicle.entity.world.load();
        for y in bounds.max.y.floor() as i32..top {
            let min = BlockPos::new(bounds.min.x.floor() as i32, y, bounds.min.z.floor() as i32);
            let max = BlockPos::new(
                bounds.max.x.ceil() as i32 - 1,
                y,
                bounds.max.z.ceil() as i32 - 1,
            );
            let mut height = 0.0f32;
            for pos in BlockPos::iterate(min, max) {
                let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
                if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                    height = height.max(world.get_fluid_height(&pos, fluid, &state));
                }
                if height >= 1.0 {
                    break;
                }
            }
            if height < 1.0 {
                return y as f32 + height;
            }
        }
        (top + 1) as f32
    }

    fn get_ground_friction(&self) -> f32 {
        let bounds = self.vehicle.entity.bounding_box.load();
        let contact = BoundingBox::new(
            bounds.min.add_raw(0.0, -WATER_OFFSET, 0.0),
            Vector3::new(bounds.max.x, bounds.min.y, bounds.max.z),
        );
        let min = BlockPos::floored_v(contact.min).offset(Vector3::new(-1, -1, -1));
        let max = BlockPos::new(
            contact.max.x.ceil() as i32,
            contact.max.y.ceil() as i32,
            contact.max.z.ceil() as i32,
        );
        let world = self.vehicle.entity.world.load();
        let mut friction = 0.0;
        let mut count = 0;
        for x in min.0.x..=max.0.x {
            for z in min.0.z..=max.0.z {
                let edges =
                    u8::from(x == min.0.x || x == max.0.x) + u8::from(z == min.0.z || z == max.0.z);
                if edges == 2 {
                    continue;
                }
                for y in min.0.y..=max.0.y {
                    if edges == 1 && (y == min.0.y || y == max.0.y) {
                        continue;
                    }
                    let pos = BlockPos::new(x, y, z);
                    let (block, state) = world.get_block_and_state(&pos);
                    if block == &Block::LILY_PAD
                        || !state
                            .get_block_collision_shapes_at(&pos)
                            .any(|shape| shape.at_pos(pos).intersects(&contact))
                    {
                        continue;
                    }
                    friction += block.slipperiness;
                    count += 1;
                }
            }
        }
        friction / count as f32
    }

    pub(super) fn float_boat(
        &self,
        old_status: Option<BoatStatus>,
        status: BoatStatus,
    ) -> Vector3<f64> {
        const DEFAULT_FRICTION: f32 = 0.9;
        let entity = &self.vehicle.entity;
        if old_status == Some(BoatStatus::InAir)
            && !matches!(status, BoatStatus::InAir | BoatStatus::OnLand)
        {
            self.water_level.store(entity.bounding_box.load().max.y);
            let target_y = f64::from(self.get_water_level_above() - entity.height()) + 0.101;
            let position = entity.pos.load();
            let bounds = entity.bounding_box.load();
            let offset = Vector3::new(0.0, target_y - position.y, 0.0);
            let target = BoundingBox::new(bounds.min + offset, bounds.max + offset);
            if entity.world.load().is_space_empty(target)
                && entity
                    .get_boat_collisions(target, self)
                    .iter()
                    .all(|shape| !shape.intersects(&target))
            {
                entity
                    .velocity
                    .store(entity.velocity.load().multiply(1.0, 0.0, 1.0));
                self.last_yd.store(0.0);
                self.status.store(Some(BoatStatus::InWater));
                return offset;
            }
            self.status.store(Some(BoatStatus::InWater));
            return Vector3::default();
        }

        let mut vertical_speed = -self.get_gravity();
        let (friction, buoyancy) = match status {
            BoatStatus::InWater => (
                DEFAULT_FRICTION,
                (self.water_level.load() - entity.pos.load().y) / f64::from(entity.height()),
            ),
            BoatStatus::UnderFlowingWater => {
                vertical_speed = -7.0e-4;
                (DEFAULT_FRICTION, 0.0)
            }
            BoatStatus::UnderWater => (0.45, f64::from(0.01f32)),
            BoatStatus::InAir => (DEFAULT_FRICTION, 0.0),
            BoatStatus::OnLand => (self.land_friction.load(), 0.0),
        };
        let velocity = entity.velocity.load();
        let mut velocity = Vector3::new(
            velocity.x * f64::from(friction),
            velocity.y + vertical_speed,
            velocity.z * f64::from(friction),
        );
        if buoyancy > 0.0 {
            velocity.y = (velocity.y + buoyancy * (DEFAULT_GRAVITY / 0.65)) * 0.75;
        }
        entity.velocity.store(velocity);
        Vector3::default()
    }

    pub(super) fn move_boat(&self, caller: &dyn EntityBase, surface_offset: Vector3<f64>) {
        let entity = &self.vehicle.entity;
        let from = entity.pos.load();
        let velocity = entity.velocity.load();
        let mut event =
            VehicleMoveEvent::new(entity.entity_id, from, from + surface_offset + velocity);
        if event.to != from
            && let Some(server) = entity.world.load().server.upgrade()
        {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            entity.velocity.store(Vector3::default());
            return;
        }
        self.last_yd.store(velocity.y);
        entity.set_pos(from + surface_offset);
        let move_from = entity.pos.load();
        entity.move_entity(caller, event.to - entity.pos.load());
        self.record_movement(move_from, entity.pos.load(), event.to - move_from);
    }

    pub(crate) fn move_from_client(
        &self,
        player: &Player,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) -> bool {
        let entity = &self.vehicle.entity;
        let from = entity.pos.load();
        let mut event = VehicleMoveEvent::new(entity.entity_id, from, position);
        if event.to != from
            && let Some(server) = entity.world.load().server.upgrade()
        {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        let movement = event.to - from;
        if event.cancelled || entity.has_new_boat_collision(self, movement) {
            self.correct_client_position(player);
            return false;
        }
        entity.set_pos(event.to);
        entity.set_rotation(yaw, pitch);
        self.last_yd.store(entity.velocity.load().y);
        self.record_movement(from, event.to, movement);
        self.position_rider(player);
        true
    }

    fn correct_client_position(&self, player: &Player) {
        let entity = &self.vehicle.entity;
        let position = entity.pos.load();
        player.try_send_client_packet(&CMoveVehicle::new(
            position.x,
            position.y,
            position.z,
            entity.yaw.load(),
            entity.pitch.load(),
        ));
    }
}
