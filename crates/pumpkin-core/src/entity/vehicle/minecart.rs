mod chest;
pub mod command_block;
pub(crate) mod container;
mod dismount;
mod furnace;
mod hopper;
mod rideable;
mod spawner;
mod tnt;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_protocol::java::server::play::SPlayerInput;
use rand::RngExt;

use crate::entity::projectile::abstract_arrow::is_abstract_arrow;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    PoweredRailLikeProperties, RailLikeProperties, RailShape, RailShapeStraight,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{atan2, wrap_degrees};

use crate::entity::vehicle::vehicle::VehicleEntity;
use crate::world::World;
use chest::ChestMinecart;
use command_block::CommandBlockMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use rideable::RideableMinecart;
use spawner::SpawnerMinecart;
use tnt::TntMinecart;

const fn get_exits(shape: RailShape) -> (Vector3<f64>, Vector3<f64>) {
    match shape {
        RailShape::NorthSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::EastWest => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::AscendingEast => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0)),
        RailShape::AscendingWest => (Vector3::new(1.0, 0.0, 0.0), Vector3::new(-1.0, 1.0, 0.0)),
        RailShape::AscendingNorth => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, -1.0)),
        RailShape::AscendingSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 1.0)),
        RailShape::SouthEast => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::SouthWest => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthWest => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthEast => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(1.0, 0.0, 0.0)),
    }
}

const GRAVITY: f64 = 0.04;
const RAIL_HEIGHT_OFFSET: f64 = 0.0625;
const ROTATION_MOVEMENT_THRESHOLD: f64 = 0.001;
const FLIP_ROTATION_THRESHOLD: f32 = 170.0;
const DEFAULT_FLIPPED_ROTATION: bool = false;
const SLOPE_ACCELERATION: f64 = 0.0078125;
const IN_WATER_SLOPE_FACTOR: f64 = 0.2;
const UNPOWERED_RAIL_BRAKE_SPEED: f64 = 0.03;
const POWERED_RAIL_MIN_SPEED: f64 = 0.01;
const POWERED_RAIL_BOOST: f64 = 0.06;
const POWERED_RAIL_START_SPEED: f64 = 0.02;
const MAX_RAIL_SPEED: f64 = 0.4;
const MAX_RAIL_SPEED_IN_WATER: f64 = 0.2;
const MAX_PROJECTED_SPEED: f64 = 2.0;
const OCCUPIED_RAIL_MOTION_FACTOR: f64 = 0.75;
const SLOPE_HEIGHT_SPEED_FACTOR: f64 = 0.05;

fn powered_rail_velocity(
    world: &World,
    pos: &BlockPos,
    shape: RailShapeStraight,
    mut velocity: Vector3<f64>,
) -> Vector3<f64> {
    let speed = velocity.horizontal_length();
    if speed > POWERED_RAIL_MIN_SPEED {
        velocity.x += velocity.x / speed * POWERED_RAIL_BOOST;
        velocity.z += velocity.z / speed * POWERED_RAIL_BOOST;
        return velocity;
    }

    let conducts = |neighbor: BlockPos| world.get_block_state(&neighbor).is_solid_block();
    match shape {
        RailShapeStraight::EastWest if conducts(pos.west()) => {
            velocity.x = POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::EastWest if conducts(pos.east()) => {
            velocity.x = -POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::NorthSouth if conducts(pos.north()) => {
            velocity.z = POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::NorthSouth if conducts(pos.south()) => {
            velocity.z = -POWERED_RAIL_START_SPEED;
        }
        _ => {}
    }
    velocity
}

fn slope_height(pos: Vector3<f64>, block_center: Vector3<f64>, uphill: Vector3<f64>) -> f64 {
    block_center.y + RAIL_HEIGHT_OFFSET + ((pos - block_center).dot(&uphill) + 0.5).clamp(0.0, 1.0)
}

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
    flipped_rotation: AtomicBool,
}

enum MinecartKind {
    Rideable(RideableMinecart),
    Chest(ChestMinecart),
    CommandBlock(CommandBlockMinecart),
    Furnace(FurnaceMinecart),
    Hopper(HopperMinecart),
    Spawner(SpawnerMinecart),
    Tnt(TntMinecart),
    Other,
}

impl MinecartEntity {
    pub(crate) const fn additional_tick_movement() -> f64 {
        // A cart can snap to a rail in the block below and cross an inclined segment.
        2.0 + SLOPE_ACCELERATION
    }

    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let kind = match entity.entity_type.id {
            id if id == EntityType::MINECART.id => MinecartKind::Rideable(RideableMinecart),
            id if id == EntityType::CHEST_MINECART.id => MinecartKind::Chest(ChestMinecart::new()),
            id if id == EntityType::COMMAND_BLOCK_MINECART.id => {
                MinecartKind::CommandBlock(CommandBlockMinecart::new())
            }
            id if id == EntityType::FURNACE_MINECART.id => {
                MinecartKind::Furnace(FurnaceMinecart::new())
            }
            id if id == EntityType::HOPPER_MINECART.id => {
                MinecartKind::Hopper(HopperMinecart::new())
            }
            id if id == EntityType::SPAWNER_MINECART.id => {
                MinecartKind::Spawner(SpawnerMinecart::new())
            }
            id if id == EntityType::TNT_MINECART.id => MinecartKind::Tnt(TntMinecart::new()),
            _ => MinecartKind::Other,
        };
        Self {
            vehicle: VehicleEntity::new(entity),
            kind,
            flipped_rotation: AtomicBool::new(DEFAULT_FLIPPED_ROTATION),
        }
    }

    fn update_rotation(&self, previous_position: Vector3<f64>, previous_yaw: f32) {
        let entity = &self.vehicle.entity;
        let movement = previous_position - entity.pos.load();
        let mut flipped = self.flipped_rotation.load(Ordering::Relaxed);
        let mut yaw = entity.yaw.load();
        // Legacy minecarts keep the same visual end facing forward when they reverse.
        if movement.horizontal_length_squared() > ROTATION_MOVEMENT_THRESHOLD {
            yaw = (atan2(movement.z, movement.x) * 180.0 / std::f64::consts::PI) as f32;
            if flipped {
                yaw += 180.0;
            }
        }
        if !(-FLIP_ROTATION_THRESHOLD..FLIP_ROTATION_THRESHOLD)
            .contains(&wrap_degrees(yaw - previous_yaw))
        {
            yaw += 180.0;
            flipped = !flipped;
            self.flipped_rotation.store(flipped, Ordering::Relaxed);
        }
        entity.set_rotation(yaw % 360.0, 0.0);
    }

    #[must_use]
    pub const fn command_block(&self) -> Option<&CommandBlockMinecart> {
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => Some(minecart),
            _ => None,
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match &self.kind {
            MinecartKind::Chest(minecart) => Some(minecart.inventory()),
            MinecartKind::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }

    const fn drop_item(&self) -> &'static Item {
        match &self.kind {
            MinecartKind::Chest(_) => &Item::CHEST_MINECART,
            MinecartKind::Furnace(_) => &Item::FURNACE_MINECART,
            MinecartKind::Hopper(_) => &Item::HOPPER_MINECART,
            MinecartKind::Tnt(_) => &Item::TNT_MINECART,
            MinecartKind::Rideable(_)
            | MinecartKind::CommandBlock(_)
            | MinecartKind::Spawner(_)
            | MinecartKind::Other => &Item::MINECART,
        }
    }

    fn rail_movement(
        &self,
        velocity: Vector3<f64>,
        in_water: bool,
        occupied: bool,
    ) -> Vector3<f64> {
        let mut max_speed = if in_water {
            MAX_RAIL_SPEED_IN_WATER
        } else {
            MAX_RAIL_SPEED
        };
        if matches!(self.kind, MinecartKind::Furnace(_)) {
            max_speed *= if in_water { 0.75 } else { 0.5 };
        }
        let scale = if occupied {
            OCCUPIED_RAIL_MOTION_FACTOR
        } else {
            1.0
        };
        Vector3::new(
            (velocity.x * scale).clamp(-max_speed, max_speed),
            0.0,
            (velocity.z * scale).clamp(-max_speed, max_speed),
        )
    }

    fn activate_rideable(&self, occupied: bool) {
        if occupied {
            let entity = self.get_entity();
            let passengers: Vec<_> = entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .map(|passenger| passenger.get_entity().entity_id)
                .collect();
            for id in passengers {
                entity.remove_passenger(id);
            }
        }
        if self.vehicle.get_hurt_time() == 0 {
            self.vehicle.set_hurt_dir(-self.vehicle.get_hurt_dir());
            self.vehicle.set_hurt_time(10);
            self.vehicle.set_damage(50.0);
            self.vehicle.send_wobble_metadata();
        }
    }

    fn push_after_moving(&self, caller: &dyn EntityBase, to_position: Vector3<f64>) {
        let entity_id = self.vehicle.entity.entity_id;
        let world = self.vehicle.entity.world.load().clone();
        if let Some(dyn_self) = world.get_entity_by_id(entity_id) {
            let should_continue = dyn_self.push_entities(caller);
            if should_continue {
                let current_pos = dyn_self.get_entity().pos.load();
                let back_motion = to_position.sub(&current_pos);
                dyn_self.get_entity().move_entity(caller, back_motion);
            }
        }
    }
}

const LOWERED_PASSENGER_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);

impl EntityBase for MinecartEntity {
    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let passenger_type = passenger.get_entity().entity_type;
        (passenger_type == &EntityType::VILLAGER || passenger_type == &EntityType::WANDERING_TRADER)
            .then_some(LOWERED_PASSENGER_ATTACHMENT)
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool(
            "FlippedRotation",
            self.flipped_rotation.load(Ordering::Relaxed),
        );
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.write_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Furnace(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.flipped_rotation.store(
            nbt.get_bool("FlippedRotation")
                .unwrap_or(DEFAULT_FLIPPED_ROTATION),
            Ordering::Relaxed,
        );
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.read_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.read_nbt(&self.vehicle.entity, nbt),
            MinecartKind::Furnace(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let previous_position = self.vehicle.entity.pos.load();
        let previous_yaw = self.vehicle.entity.yaw.load();
        self.vehicle.tick();
        match &self.kind {
            MinecartKind::Furnace(minecart) => minecart.tick(&self.vehicle.entity),
            MinecartKind::CommandBlock(minecart) => minecart.step_tick(),
            _ => {}
        }

        let world = self.vehicle.entity.world.load();
        let pos = self.vehicle.entity.pos.load();
        let mut block_pos = BlockPos(Vector3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ));

        let (mut block, mut state_id) = world.get_block_and_state_id(&block_pos);

        let mut is_powered_rail = block.id == Block::POWERED_RAIL.id;
        let mut is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
        let mut is_on_rails = is_powered_rail
            || is_activator_rail
            || block.id == Block::RAIL.id
            || block.id == Block::DETECTOR_RAIL.id;

        // If not on rails at current Y level, check the block directly below
        if !is_on_rails {
            let below_block_pos = BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y - 1,
                block_pos.0.z,
            ));
            let (below_block, below_state_id) = world.get_block_and_state_id(&below_block_pos);
            if below_block.id == Block::RAIL.id
                || below_block.id == Block::POWERED_RAIL.id
                || below_block.id == Block::DETECTOR_RAIL.id
                || below_block.id == Block::ACTIVATOR_RAIL.id
            {
                block_pos = below_block_pos;
                block = below_block;
                state_id = below_state_id;
                is_powered_rail = block.id == Block::POWERED_RAIL.id;
                is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
                is_on_rails = true;
            }
        }

        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id);
            let powered = props.powered;

            if is_activator_rail && let MinecartKind::Hopper(minecart) = &self.kind {
                minecart.set_enabled(!powered);
            }
        } else if block.id == Block::DETECTOR_RAIL.id
            && let Some(server) = world.server.upgrade()
        {
            world.block_registry.on_entity_collision(
                block,
                &world,
                self,
                &block_pos,
                world.get_block_state(&block_pos),
                &server,
            );
        }

        let mut powered_rail = None;
        let mut brake_on_rail = false;
        let mut activate_rideable = false;
        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id);
            let powered = props.powered;

            if powered {
                if is_powered_rail {
                    powered_rail = Some(props.shape);
                } else if is_activator_rail {
                    match &self.kind {
                        MinecartKind::CommandBlock(minecart) => {
                            minecart.activate(&self.vehicle.entity);
                        }
                        MinecartKind::Tnt(minecart) => {
                            minecart.prime(&self.vehicle.entity, 80);
                        }
                        MinecartKind::Rideable(_) => activate_rideable = true,
                        _ => {}
                    }
                }
            } else if is_powered_rail {
                brake_on_rail = true;
            }
        }

        if let MinecartKind::Tnt(minecart) = &self.kind
            && minecart.tick(self)
        {
            return;
        }

        let mut velocity = self.vehicle.entity.velocity.load();

        let mut has_driver = false;
        let mut driver_input = 0;
        let mut driver_yaw = 0.0f32;

        if let Ok(passengers) = self.vehicle.entity.passengers.try_lock()
            && let Some(passenger) = passengers.first()
            && let Some(player) = passenger.get_player()
        {
            driver_input = player.last_input.load(Ordering::Relaxed);
            driver_yaw = player.get_entity().yaw.load();
            has_driver = true;
        }

        if has_driver && is_on_rails {
            let forward = driver_input & SPlayerInput::FORWARD != 0;
            let backward = driver_input & SPlayerInput::BACKWARD != 0;

            let mut force_dir = Vector3::new(0.0, 0.0, 0.0);
            if forward {
                let yaw_rad = f64::from(driver_yaw).to_radians();
                force_dir.x = -yaw_rad.sin();
                force_dir.z = yaw_rad.cos();
            } else if backward {
                let yaw_rad = f64::from(driver_yaw).to_radians();
                force_dir.x = yaw_rad.sin();
                force_dir.z = -yaw_rad.cos();
            }

            if forward || backward {
                brake_on_rail &= velocity.horizontal_length_squared() >= 0.01;
                velocity.x += force_dir.x * 0.02;
                velocity.z += force_dir.z * 0.02;

                let speed = velocity.x.hypot(velocity.z);
                if speed > 0.15 {
                    #[allow(clippy::suboptimal_flops)]
                    let old_speed = self
                        .vehicle
                        .entity
                        .velocity
                        .load()
                        .x
                        .hypot(self.vehicle.entity.velocity.load().z);

                    let max_speed = old_speed.clamp(0.15, 0.4);
                    if speed > max_speed {
                        velocity.x = (velocity.x / speed) * max_speed;
                        velocity.z = (velocity.z / speed) * max_speed;
                    }
                }
                self.vehicle.entity.velocity.store(velocity);
                self.vehicle.entity.send_velocity();
            }
        }

        let mut velocity = self.vehicle.entity.velocity.load();
        let occupied = is_on_rails && self.get_entity().has_passengers();
        let in_water = is_on_rails && {
            let bounds = self.get_entity().bounding_box.load().expand_all(-0.001);
            BlockPos::iterate(bounds.min_block_pos(), bounds.max_block_pos()).any(|pos| {
                let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
                fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)
                    && f64::from(pos.0.y) + f64::from(world.get_fluid_height(&pos, fluid, &state))
                        >= bounds.min.y
            })
        };
        let mut slope = None;
        if is_on_rails {
            let shape = if block.id == Block::RAIL.id {
                let props = RailLikeProperties::from_state_id(state_id);
                props.shape
            } else {
                let props = PoweredRailLikeProperties::from_state_id(state_id);
                match props.shape {
                    RailShapeStraight::NorthSouth => RailShape::NorthSouth,
                    RailShapeStraight::EastWest => RailShape::EastWest,
                    RailShapeStraight::AscendingEast => RailShape::AscendingEast,
                    RailShapeStraight::AscendingWest => RailShape::AscendingWest,
                    RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
                    RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
                }
            };

            let pos = self.vehicle.entity.pos.load();
            let block_center_bottom = Vector3::new(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y),
                f64::from(block_pos.0.z) + 0.5,
            );

            let (exit0, exit1) = get_exits(shape);
            if exit0.y != exit1.y {
                let uphill = Vector3::new(exit1.x, 0.0, exit1.z);
                let acceleration = if in_water {
                    SLOPE_ACCELERATION * IN_WATER_SLOPE_FACTOR
                } else {
                    SLOPE_ACCELERATION
                };
                velocity = velocity - uphill * acceleration;
                slope = Some((block_center_bottom, uphill));
            }
            let exit0 = exit0.multiply(0.5, 0.5, 0.5);
            let exit1 = exit1.multiply(0.5, 0.5, 0.5);

            let in_corner = exit0.x != exit1.x && exit0.z != exit1.z;
            let mut target_position = pos;

            if in_corner {
                let from0to1 = exit1 - exit0;
                let from0topos = pos - block_center_bottom - exit0;
                let dot_num = from0to1.dot(&from0topos);
                let dot_den = from0to1.dot(&from0to1);
                if dot_den != 0.0 {
                    let travel_vector_from0 =
                        from0to1.multiply(dot_num / dot_den, dot_num / dot_den, dot_num / dot_den);
                    target_position = block_center_bottom.add(&exit0).add(&travel_vector_from0);
                }
            } else {
                let z_snap = (exit0.x - exit1.x).abs() > 1e-5;
                let x_snap = (exit0.z - exit1.z).abs() > 1e-5;
                if x_snap {
                    target_position.x = block_center_bottom.x;
                }
                if z_snap {
                    target_position.z = block_center_bottom.z;
                }
            }

            target_position.y = if slope.is_some() {
                // Clear the higher end while moving, then put the cart back onto the rail.
                block_center_bottom.y + 1.0
            } else {
                block_center_bottom.y + RAIL_HEIGHT_OFFSET
            };
            self.vehicle.entity.set_pos(target_position);

            let horizontal_in_direction = Vector3::new(exit1.x, 0.0, exit1.z);
            let mut horizontal_out_direction = Vector3::new(exit0.x, 0.0, exit0.z);

            if velocity.dot(&horizontal_out_direction) < velocity.dot(&horizontal_in_direction) {
                horizontal_out_direction = horizontal_in_direction;
            }

            let out_position = block_center_bottom.add(&horizontal_out_direction).add(
                &horizontal_out_direction
                    .normalize()
                    .multiply(1e-5, 1e-5, 1e-5),
            );

            let mut towards_out = out_position - target_position;
            towards_out.y = 0.0;
            let towards_length = towards_out.length();
            if towards_length > 1e-5 {
                towards_out = towards_out.normalize();
                velocity = towards_out * velocity.horizontal_length().min(MAX_PROJECTED_SPEED);
            }

            velocity.y = 0.0;
            if brake_on_rail {
                velocity = if velocity.horizontal_length() < UNPOWERED_RAIL_BRAKE_SPEED {
                    Vector3::default()
                } else {
                    velocity * 0.5
                };
            }
            self.vehicle.entity.velocity.store(velocity);
        } else if !self.vehicle.entity.on_ground.load(Ordering::Relaxed) {
            velocity.y -= GRAVITY;
            self.vehicle.entity.velocity.store(velocity);
        }

        let motion = if is_on_rails {
            self.rail_movement(velocity, in_water, occupied)
        } else {
            velocity
        };
        let moved = if is_on_rails {
            motion != Vector3::default()
        } else {
            motion.length() > 0.001
        };
        let height_change = if let Some((rail_center, uphill)) = slope {
            let entity = self.get_entity();
            let previous_height = slope_height(entity.pos.load(), rail_center, uphill);
            let intended = entity.pos.load() + motion;
            if moved {
                entity.move_entity(caller, motion);
            }
            let mut position = entity.pos.load();
            position.y = slope_height(position, rail_center, uphill);
            entity.set_pos(position);
            if moved {
                self.push_after_moving(caller, Vector3::new(intended.x, position.y, intended.z));
            }
            previous_height - position.y
        } else {
            if moved {
                self.move_entity(caller, motion);
            }
            0.0
        };

        if moved {
            if let MinecartKind::Tnt(minecart) = &self.kind
                && self
                    .vehicle
                    .entity
                    .horizontal_collision
                    .load(Ordering::Relaxed)
                && motion.horizontal_length_squared() >= 0.01
            {
                minecart.explode(self, motion.horizontal_length_squared(), None);
                return;
            }

            #[allow(clippy::useless_let_if_seq)]
            let mut friction = 0.95; // Vanilla minecart air drag

            if is_on_rails {
                // The movement solver clips blocked axes and contact pushes can alter stored speed.
                velocity = self.get_entity().velocity.load();
                friction = if occupied { 0.997 } else { 0.96 };
                if in_water {
                    friction *= 0.95;
                }
            } else {
                let below_block_pos = BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y - 1,
                    block_pos.0.z,
                ));
                let below_block = world.get_block(&below_block_pos);

                let is_on_ground = self.vehicle.entity.on_ground.load(Ordering::Relaxed)
                    || (below_block.id != Block::AIR.id
                        && below_block.id != Block::WATER.id
                        && below_block.id != Block::LAVA.id);
                let is_in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed)
                    || below_block.id == Block::WATER.id;

                if is_on_ground {
                    friction = 0.5;
                } else if is_in_water {
                    friction = 0.95;
                }
            }

            let mut next_vel = if is_on_rails && let MinecartKind::Furnace(minecart) = &self.kind {
                minecart.velocity(velocity, in_water)
            } else if is_on_rails && let Some(inventory) = self.container() {
                container::velocity(inventory, velocity, in_water)
            } else {
                velocity.multiply(friction, friction, friction)
            };
            let speed = next_vel.horizontal_length();
            if height_change != 0.0 && speed > 0.0 {
                next_vel = next_vel * ((speed + height_change * SLOPE_HEIGHT_SPEED_FACTOR) / speed);
            }
            if !is_on_rails && next_vel.length() < 0.005 {
                next_vel = Vector3::new(0.0, 0.0, 0.0);
            }
            self.vehicle.entity.velocity.store(next_vel);
            if next_vel.length_squared() == 0.0 {
                self.vehicle.entity.send_velocity();
            }
        }

        if let Some(shape) = powered_rail {
            // Legacy minecarts apply powered-rail acceleration after moving along the track.
            let current = self.vehicle.entity.velocity.load();
            let boosted = powered_rail_velocity(&world, &block_pos, shape, current);
            if boosted != current {
                self.vehicle.entity.velocity.store(boosted);
                self.vehicle.entity.send_velocity();
            }
        }
        if activate_rideable {
            self.activate_rideable(occupied);
        }
        self.update_rotation(previous_position, previous_yaw);

        match &self.kind {
            MinecartKind::Hopper(minecart) => minecart.tick(&self.vehicle.entity),
            MinecartKind::Spawner(minecart) => minecart.tick(&self.vehicle.entity),
            _ => {}
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn has_container_inventory(&self) -> bool {
        self.container().is_some()
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn Inventory>> {
        let container = self.container()?;
        if container.has_loot_table() {
            let world = self.vehicle.entity.world.load();
            container.unpack_loot(&crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(self.vehicle.entity.pos.load()),
                ..Default::default()
            });
        }
        Some(container.clone() as Arc<dyn Inventory>)
    }

    fn is_pushable(&self) -> bool {
        true
    }

    #[allow(clippy::too_many_lines)]
    fn push(&self, entity: &dyn EntityBase) {
        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
        {
            return;
        }

        {
            let passengers = self_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == other_entity.entity_id)
            {
                return;
            }
        }
        {
            let passengers = other_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == self_entity.entity_id)
            {
                return;
            }
        }

        let mut xa = other_entity.pos.load().x - self_entity.pos.load().x;
        let mut za = other_entity.pos.load().z - self_entity.pos.load().z;
        let mut dd = xa * xa + za * za;
        if dd >= 1.0E-4 {
            dd = dd.sqrt();
            xa /= dd;
            za /= dd;
            let mut pow = 1.0 / dd;
            if pow > 1.0 {
                pow = 1.0;
            }
            xa *= pow;
            za *= pow;
            xa *= 0.1;
            za *= 0.1;
            xa *= 0.5;
            za *= 0.5;

            let is_other_minecart = other_entity.entity_type.id == EntityType::MINECART.id
                || other_entity.entity_type.id == EntityType::CHEST_MINECART.id
                || other_entity.entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
                || other_entity.entity_type.id == EntityType::FURNACE_MINECART.id
                || other_entity.entity_type.id == EntityType::HOPPER_MINECART.id
                || other_entity.entity_type.id == EntityType::SPAWNER_MINECART.id
                || other_entity.entity_type.id == EntityType::TNT_MINECART.id;

            if is_other_minecart {
                let xo = self_entity.velocity.load().x;
                let zo = self_entity.velocity.load().z;

                let dir = Vector3::new(xo, 0.0, zo).normalize();
                let facing = Vector3::new(
                    f64::from(self_entity.yaw.load().to_radians().cos()),
                    0.0,
                    f64::from(self_entity.yaw.load().to_radians().sin()),
                )
                .normalize();

                let dot = dir.dot(&facing).abs();
                if dot >= 0.8 {
                    let vel = self_entity.velocity.load();
                    let ovel = other_entity.velocity.load();

                    let is_self_furnace =
                        self_entity.entity_type.id == EntityType::FURNACE_MINECART.id;
                    let is_other_furnace =
                        other_entity.entity_type.id == EntityType::FURNACE_MINECART.id;

                    if is_other_furnace && !is_self_furnace {
                        self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                        let mut new_self_vel = self_entity.velocity.load();
                        new_self_vel.x += ovel.x - xa;
                        new_self_vel.z += ovel.z - za;
                        self_entity.velocity.store(new_self_vel);
                        self_entity.send_velocity();

                        other_entity.velocity.store(ovel.multiply(0.95, 1.0, 0.95));
                        other_entity.send_velocity();
                    } else if !is_other_furnace && is_self_furnace {
                        other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                        let mut new_other_vel = other_entity.velocity.load();
                        new_other_vel.x += vel.x + xa;
                        new_other_vel.z += vel.z + za;
                        other_entity.velocity.store(new_other_vel);
                        other_entity.send_velocity();

                        self_entity.velocity.store(vel.multiply(0.95, 1.0, 0.95));
                        self_entity.send_velocity();
                    } else {
                        #[allow(clippy::manual_midpoint)]
                        let xdd = (ovel.x + vel.x) / 2.0;
                        #[allow(clippy::manual_midpoint)]
                        let zdd = (ovel.z + vel.z) / 2.0;

                        self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                        let mut new_self_vel = self_entity.velocity.load();
                        new_self_vel.x += xdd - xa;
                        new_self_vel.z += zdd - za;
                        self_entity.velocity.store(new_self_vel);
                        self_entity.send_velocity();

                        other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                        let mut new_other_vel = other_entity.velocity.load();
                        new_other_vel.x += xdd + xa;
                        new_other_vel.z += zdd + za;
                        other_entity.velocity.store(new_other_vel);
                        other_entity.send_velocity();
                    }
                }
            } else {
                // Vanilla `AbstractMinecart.push`
                self_entity.push_by(Vector3::new(-xa, 0.0, -za));
                other_entity.push_by(Vector3::new(xa / 4.0, 0.0, za / 4.0));
            }
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn init_data_tracker(&self) {
        self.vehicle.send_wobble_metadata();
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            MinecartKind::Furnace(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            _ => {}
        }
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && let Some(projectile) = source
            && is_abstract_arrow(projectile.get_entity().entity_type)
            && projectile.get_entity().is_on_fire()
        {
            let world = self.vehicle.entity.world.load();
            let cause =
                cause.and_then(|cause| world.get_entity_by_id(cause.get_entity().entity_id));
            minecart.explode(
                self,
                projectile.get_entity().velocity.load().length_squared(),
                cause,
            );
            if self.vehicle.entity.is_removed() {
                return true;
            }
        }

        let will_break = self.vehicle.entity.is_alive()
            && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && will_break
            && !creative
        {
            let velocity = self.vehicle.entity.velocity.load();
            let speed_squared = velocity.x.mul_add(velocity.x, velocity.z * velocity.z);
            let ignites = damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE)
                || damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION)
                || self.vehicle.entity.fire_ticks.load(Ordering::Relaxed) > 0;
            if ignites || speed_squared >= 0.01 {
                self.vehicle.apply_damage_wobble(amount);
                let fuse = rand::rng().random_range(0..20) + rand::rng().random_range(0..20);
                if self
                    .vehicle
                    .entity
                    .world
                    .load()
                    .level_info
                    .load()
                    .game_rules
                    .tnt_explodes
                {
                    minecart.prime(&self.vehicle.entity, fuse);
                } else {
                    minecart.set_fuse(fuse);
                }
                return true;
            }
        }

        let damaged = self.vehicle.damage_with_context(amount, source);

        if will_break && !creative && self.vehicle.entity.is_removed() {
            let world = self.vehicle.entity.world.load();
            if world.level_info.load().game_rules.entity_drops {
                let position = self.vehicle.entity.block_pos.load();
                if let Some(container) = self.container()
                    && container.claim_drops()
                {
                    container.unpack_loot(&crate::world::loot::LootContextParameters {
                        world: Some(&world),
                        position: Some(self.vehicle.entity.pos.load()),
                        ..Default::default()
                    });
                    let inventory: Arc<dyn Inventory> = container.clone();
                    world.scatter_inventory(&position, &inventory);
                }
                let mut stack = ItemStack::new(1, self.drop_item());
                if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                    stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                        name: custom_name.clone(),
                    });
                }
                world.drop_stack(&position, stack);
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::CommandBlock(_) => CommandBlockMinecart::interact(player),
            MinecartKind::Furnace(minecart) => {
                minecart.interact(&self.vehicle.entity, player, item_stack)
            }
            MinecartKind::Hopper(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::Rideable(_) => RideableMinecart::interact(&self.vehicle.entity, player),
            MinecartKind::Spawner(_) | MinecartKind::Tnt(_) | MinecartKind::Other => false,
        }
    }

    fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        let to_position = self.vehicle.entity.pos.load().add(&motion);
        self.vehicle.entity.move_entity(caller, motion);
        self.push_after_moving(caller, to_position);
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::zombie::zombie::ZombieEntity;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::text::TextComponent;
    use pumpkin_world::chunk::ChunkData;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn powered_rail_kick_uses_conducting_neighbors() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = ChunkData::empty_sync(0, 0);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), chunk.clone());
        let pos = BlockPos::new(8, 64, 8);
        let stopped = Vector3::default();

        chunk.set_block_absolute_y(8, 64, 7, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(9, 64, 8, Block::GLASS.default_state.id);
        assert_eq!(
            powered_rail_velocity(&world, &pos, RailShapeStraight::NorthSouth, stopped),
            Vector3::new(0.0, 0.0, 0.02),
        );
        assert_eq!(
            powered_rail_velocity(&world, &pos, RailShapeStraight::EastWest, stopped),
            stopped,
        );
        chunk.set_block_absolute_y(9, 64, 8, Block::STONE.default_state.id);
        assert_eq!(
            powered_rail_velocity(&world, &pos, RailShapeStraight::EastWest, stopped),
            Vector3::new(-0.02, 0.0, 0.0),
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn powered_rail_boost_and_displacement_are_independent() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = BlockPos::new(8, 64, 8);
        let minecart = MinecartEntity::new(Entity::new(
            world.clone(),
            pos.to_centered_f64(),
            &EntityType::MINECART,
        ));
        let boosted = powered_rail_velocity(
            &world,
            &pos,
            RailShapeStraight::EastWest,
            Vector3::new(0.4, 0.0, 0.0),
        );
        assert!((boosted.x - 0.46).abs() < f64::EPSILON);
        assert_eq!(minecart.rail_movement(boosted, false, false).x, 0.4);
        assert_eq!(minecart.rail_movement(boosted, true, false).x, 0.2);
        let occupied_motion = minecart.rail_movement(boosted, false, true);
        assert!((occupied_motion.x - 0.345).abs() < f64::EPSILON);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn minecart_dismount_tracks_reversing_motion_and_checks_the_other_side() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = ChunkData::empty_sync(0, 0);
        for x in 7..=9 {
            for z in 7..=9 {
                chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
            }
        }
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), chunk.clone());
        let position = Vector3::new(8.5, 64.0625, 8.5);
        let minecart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            position,
            &EntityType::MINECART,
        )));
        let rider = ZombieEntity::new(Entity::new(world.clone(), position, &EntityType::ZOMBIE));
        assert!(world.spawn_entity(minecart.clone()));
        assert!(world.spawn_entity(rider.clone()));

        // When a yaw-0 cart rolls east, it keeps its visible orientation and flips internally.
        minecart.update_rotation(position.add_raw(-0.2, 0.0, 0.0), 0.0);
        let mut saved = NbtCompound::new();
        minecart.write_custom_nbt(&mut saved);
        assert_eq!(saved.get_bool("FlippedRotation"), Some(true));
        assert_eq!(
            MinecartEntity::get_dismount_location_for_passenger(
                minecart.get_entity(),
                rider.as_ref()
            ),
            Vector3::new(8.5, 64.0, 9.5)
        );

        minecart.update_rotation(position.add_raw(0.2, 0.0, 0.0), 0.0);
        assert_eq!(
            MinecartEntity::get_dismount_location_for_passenger(
                minecart.get_entity(),
                rider.as_ref()
            ),
            Vector3::new(8.5, 64.0, 7.5)
        );
        minecart.read_custom_nbt(&saved);
        assert_eq!(
            MinecartEntity::get_dismount_location_for_passenger(
                minecart.get_entity(),
                rider.as_ref()
            ),
            Vector3::new(8.5, 64.0, 9.5)
        );
        chunk.set_block_absolute_y(8, 64, 9, Block::STONE.default_state.id);
        let vehicle = minecart.get_entity();
        let passenger = rider.get_entity();
        vehicle.add_passenger(minecart.clone(), rider.clone());
        assert!(passenger.has_vehicle());
        vehicle.remove_passenger(passenger.entity_id);
        assert!(!passenger.has_vehicle());
        assert_eq!(passenger.pos.load(), Vector3::new(8.5, 64.0, 7.5));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn command_block_minecart_nbt_and_activation_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COMMAND_BLOCK_MINECART,
        ));

        let mut nbt = NbtCompound::new();
        nbt.put_string("Command", "Searge".to_string());
        nbt.put_bool("TrackOutput", true);
        nbt.put_long("LastExecution", 12);
        cart.read_nbt_non_mut(&nbt);

        let Some(cb) = cart.command_block() else {
            return Err("expected CommandBlockMinecart".into());
        };
        assert_eq!(cb.get_command(), "Searge");
        assert_eq!(cb.get_success_count(), 0);

        // Fewer than 4 ticks elapsed since `last_activated` (0): must not activate.
        for _ in 0..3 {
            cb.step_tick();
            cb.activate(&cart.vehicle.entity);
        }
        assert_eq!(cb.get_success_count(), 0);

        // Tick 4 reaches ACTIVATION_DELAY (4) and triggers the command.
        cb.step_tick();
        cb.activate(&cart.vehicle.entity);
        assert_eq!(cb.get_success_count(), 1);
        assert_eq!(
            cb.get_last_output_or_empty(),
            TextComponent::text("#itzlipofutzli")
        );

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_string("Command"), Some("Searge"));
        assert_eq!(saved.get_int("SuccessCount"), Some(1));
        assert_eq!(saved.get_bool("TrackOutput"), Some(true));
        assert!(saved.get("LastOutput").is_some());

        // Setting TrackOutput=false clears LastOutput on load and omits it on save.
        let mut no_track = saved.clone();
        no_track.put_bool("TrackOutput", false);
        cart.read_nbt_non_mut(&no_track);
        let mut saved_no_track = NbtCompound::new();
        cart.write_nbt(&mut saved_no_track);
        assert!(saved_no_track.get("LastOutput").is_none());

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn spawner_minecart_nbt_roundtrip() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SPAWNER_MINECART,
        ));

        let mut spawn_entity = NbtCompound::new();
        spawn_entity.put_string("id", "minecraft:pig".to_string());
        let mut spawn_data = NbtCompound::new();
        spawn_data.put_compound("entity", spawn_entity);

        let mut nbt = NbtCompound::new();
        nbt.put_short("Delay", 45);
        nbt.put_short("MinSpawnDelay", 100);
        nbt.put_short("MaxSpawnDelay", 300);
        nbt.put_short("SpawnCount", 2);
        nbt.put_short("SpawnRange", 3);
        nbt.put_short("MaxNearbyEntities", 5);
        nbt.put_short("RequiredPlayerRange", 12);
        nbt.put_compound("SpawnData", spawn_data);

        cart.read_nbt_non_mut(&nbt);

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_short("Delay"), Some(45));
        assert_eq!(saved.get_short("MinSpawnDelay"), Some(100));
        assert_eq!(saved.get_short("MaxSpawnDelay"), Some(300));
        assert_eq!(saved.get_short("SpawnCount"), Some(2));
        assert_eq!(saved.get_short("SpawnRange"), Some(3));
        assert_eq!(saved.get_short("MaxNearbyEntities"), Some(5));
        assert_eq!(saved.get_short("RequiredPlayerRange"), Some(12));
        assert_eq!(
            saved
                .get_compound("SpawnData")
                .and_then(|d| d.get_compound("entity"))
                .and_then(|e| e.get_string("id")),
            Some("minecraft:pig")
        );

        world.level.shutdown().await;
        Ok(())
    }
}
