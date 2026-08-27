mod chest;
mod container;
mod furnace;
mod hopper;
mod rideable;
mod tnt;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_protocol::java::server::play::SPlayerInput;
use rand::RngExt;

use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
    world::World,
};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, PoweredRailLikeProperties, RailShape};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_world::inventory::Inventory;

use crate::entity::vehicle::vehicle::VehicleEntity;
use chest::ChestMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use rideable::RideableMinecart;
use tnt::TntMinecart;

/// Vanilla `AbstractMinecart` entity types. Scopes [`rail_collision_ignore_positions`]
/// the same way `CollisionContext.of` matches `case AbstractMinecart` (Java class, not a block).
#[must_use]
pub const fn is_minecart(entity_type: &EntityType) -> bool {
    entity_type.id == EntityType::MINECART.id
        || entity_type.id == EntityType::CHEST_MINECART.id
        || entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
        || entity_type.id == EntityType::FURNACE_MINECART.id
        || entity_type.id == EntityType::HOPPER_MINECART.id
        || entity_type.id == EntityType::SPAWNER_MINECART.id
        || entity_type.id == EntityType::TNT_MINECART.id
}

/// Vanilla `MinecartCollisionContext.setupContext`: ignore the cell under the rail, and on a
/// slope the cell the rail climbs into.
///
/// Recomputed from current position (a piston-carried cart moves through
/// `Entity::move_entity_piston`, not this `tick`).
pub fn rail_collision_ignore_positions(
    world: &World,
    entity_pos: Vector3<f64>,
) -> [Option<BlockPos>; 2] {
    use pumpkin_data::block_properties::RailLikeProperties;
    use pumpkin_data::block_properties::{RailShape, RailShapeStraight};

    let mut block_pos = BlockPos(Vector3::new(
        entity_pos.x.floor() as i32,
        entity_pos.y.floor() as i32,
        entity_pos.z.floor() as i32,
    ));

    // Vanilla `getCurrentBlockPosOrRailBelow`: drop one cell if the cell below is a rail,
    // then test that cell. Stacked rails: vanilla picks the lower.
    let below_block_pos = BlockPos(Vector3::new(
        block_pos.0.x,
        block_pos.0.y - 1,
        block_pos.0.z,
    ));
    if world
        .get_block(&below_block_pos)
        .has_tag(&tag::Block::MINECRAFT_RAILS)
    {
        block_pos = below_block_pos;
    }

    let block = world.get_block(&block_pos);
    if !block.has_tag(&tag::Block::MINECRAFT_RAILS) {
        return [None, None];
    }

    let state_id = world.get_block_state_id(&block_pos);
    let shape = if block.id == Block::RAIL.id {
        RailLikeProperties::from_state_id(state_id, block).shape
    } else {
        match PoweredRailLikeProperties::from_state_id(state_id, block).shape {
            RailShapeStraight::NorthSouth => RailShape::NorthSouth,
            RailShapeStraight::EastWest => RailShape::EastWest,
            RailShapeStraight::AscendingEast => RailShape::AscendingEast,
            RailShapeStraight::AscendingWest => RailShape::AscendingWest,
            RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
            RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
        }
    };

    let below_pos = BlockPos(Vector3::new(
        block_pos.0.x,
        block_pos.0.y - 1,
        block_pos.0.z,
    ));
    let slope_ignore_pos = match shape {
        RailShape::AscendingEast => Some(BlockPos(Vector3::new(
            block_pos.0.x + 1,
            block_pos.0.y,
            block_pos.0.z,
        ))),
        RailShape::AscendingWest => Some(BlockPos(Vector3::new(
            block_pos.0.x - 1,
            block_pos.0.y,
            block_pos.0.z,
        ))),
        RailShape::AscendingNorth => Some(BlockPos(Vector3::new(
            block_pos.0.x,
            block_pos.0.y,
            block_pos.0.z - 1,
        ))),
        RailShape::AscendingSouth => Some(BlockPos(Vector3::new(
            block_pos.0.x,
            block_pos.0.y,
            block_pos.0.z + 1,
        ))),
        _ => None,
    };

    [Some(below_pos), slope_ignore_pos]
}

const fn get_exits(
    shape: pumpkin_data::block_properties::RailShape,
) -> (Vector3<f64>, Vector3<f64>) {
    use pumpkin_data::block_properties::RailShape;
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

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
    /// One-time `tick_block_collisions` while at rest. Cleared when velocity is nonzero.
    block_collisions_checked_at_rest: AtomicBool,
}

enum MinecartKind {
    Rideable(RideableMinecart),
    Chest(ChestMinecart),
    Furnace(FurnaceMinecart),
    Hopper(HopperMinecart),
    Tnt(TntMinecart),
    Other,
}

impl MinecartEntity {
    pub fn new(entity: Entity) -> Self {
        let kind = match entity.entity_type.id {
            id if id == EntityType::MINECART.id => MinecartKind::Rideable(RideableMinecart),
            id if id == EntityType::CHEST_MINECART.id => MinecartKind::Chest(ChestMinecart::new()),
            id if id == EntityType::FURNACE_MINECART.id => {
                MinecartKind::Furnace(FurnaceMinecart::new())
            }
            id if id == EntityType::HOPPER_MINECART.id => {
                MinecartKind::Hopper(HopperMinecart::new())
            }
            id if id == EntityType::TNT_MINECART.id => MinecartKind::Tnt(TntMinecart::new()),
            _ => MinecartKind::Other,
        };
        Self {
            vehicle: VehicleEntity::new(entity),
            kind,
            block_collisions_checked_at_rest: AtomicBool::new(false),
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match &self.kind {
            MinecartKind::Chest(minecart) => Some(minecart.inventory()),
            MinecartKind::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }

    /// Vanilla `DetectorRailBlock.getAnalogOutputSignal`: fill of a chest/hopper minecart.
    /// Plain, furnace, TNT, or a player: `0` (they still power the rail's redstone).
    pub fn container_comparator_output(&self) -> Option<u8> {
        let inventory = self.container()?;
        Some(crate::block::calculate_comparator_output(
            inventory.as_ref(),
        ))
    }

    /// Cargo changed since last call. Always `false` without a container ([`MinecartInventory::take_dirty`]).
    pub fn take_container_dirty(&self) -> bool {
        self.container()
            .is_some_and(|inventory| inventory.take_dirty())
    }

    const fn drop_item(&self) -> Option<&'static Item> {
        match &self.kind {
            MinecartKind::Chest(_) => Some(&Item::CHEST_MINECART),
            MinecartKind::Furnace(_) => Some(&Item::FURNACE_MINECART),
            MinecartKind::Hopper(_) => Some(&Item::HOPPER_MINECART),
            MinecartKind::Tnt(_) => Some(&Item::TNT_MINECART),
            _ => None,
        }
    }

    /// Vanilla `MinecartBehavior.getMaxSpeed`: `max_minecart_speed` game rule in blocks/s
    /// (default 8, 0.4 per tick), halved in water. Caps the step `moveAlongTrack` /
    /// `comeOffTrack` hands to `move`.
    fn max_speed(&self) -> f64 {
        let world = self.vehicle.entity.world.load();
        let per_second = world.level_info.load().game_rules.max_minecart_speed;
        let in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed);
        per_second as f64 * if in_water { 0.5 } else { 1.0 } / 20.0
    }

    /// Vanilla `comeOffTrack` `onGround()`: `Entity::on_ground`, or a solid block below
    /// (`on_ground` is only set by a downward collision).
    fn grounded_off_rail(&self, world: &World, block_pos: BlockPos) -> bool {
        if self.vehicle.entity.on_ground.load(Ordering::Relaxed) {
            return true;
        }
        let below = world.get_block(&BlockPos(Vector3::new(
            block_pos.0.x,
            block_pos.0.y - 1,
            block_pos.0.z,
        )));
        below.id != Block::AIR.id && below.id != Block::WATER.id && below.id != Block::LAVA.id
    }

    /// Vanilla `ActivatorRailBlock`: a powered activator rail ejects every rider.
    ///
    /// `Entity::remove_passenger` is still async (it fires the dismount events and has to get
    /// the `CSetPassengers`/teleport pair to the dismounting player in order), so the ejection
    /// is handed to the runtime instead of blocking the tick. `caller` keeps the cart alive
    /// until the task runs.
    fn dismount_all_passengers(&self, caller: &dyn EntityBase) {
        let passenger_ids: Vec<i32> = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|passenger| passenger.get_entity().entity_id)
            .collect();

        if passenger_ids.is_empty() {
            return;
        }

        let world = caller.get_entity().world.load();
        let vehicle_id = caller.get_entity().entity_id;
        let Some(vehicle) = world.get_entity_by_id(vehicle_id) else {
            return;
        };
        tokio::spawn(async move {
            for passenger_id in passenger_ids {
                vehicle.get_entity().remove_passenger(passenger_id).await;
            }
        });
    }
}

impl EntityBase for MinecartEntity {
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Furnace(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Furnace(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.vehicle.tick();
        if let MinecartKind::Furnace(minecart) = &self.kind {
            minecart.tick(&self.vehicle.entity);
        }

        let world = self.vehicle.entity.world.load();
        let pos = self.vehicle.entity.pos.load();
        let mut block_pos = BlockPos(Vector3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ));

        // While the rail is a `MOVING_PISTON` placeholder the cart is off rails.
        // Same descent as `rail_collision_ignore_positions` (`ignoreBelow`).
        let below_block_pos = BlockPos(Vector3::new(
            block_pos.0.x,
            block_pos.0.y - 1,
            block_pos.0.z,
        ));
        if world
            .get_block(&below_block_pos)
            .has_tag(&tag::Block::MINECRAFT_RAILS)
        {
            block_pos = below_block_pos;
        }

        let (block, state_id) = world.get_block_and_state_id(&block_pos);

        let is_powered_rail = block.id == Block::POWERED_RAIL.id;
        let is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
        let is_on_rails = block.has_tag(&tag::Block::MINECRAFT_RAILS);

        // Vanilla `moveAlongTrack`: `halt_track` brakes before the move, `power_track`
        // boosts after the move and after friction.
        let mut power_track = false;
        let mut halt_track = false;

        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id, block);
            let powered = props.powered;

            if is_activator_rail && let MinecartKind::Hopper(minecart) = &self.kind {
                minecart.set_enabled(!powered);
            }

            if is_powered_rail {
                power_track = powered;
                halt_track = !powered;
            }

            if powered && is_activator_rail {
                match &self.kind {
                    MinecartKind::Tnt(minecart) => {
                        minecart.prime(&self.vehicle.entity, 80);
                    }
                    MinecartKind::Rideable(_) => {
                        self.dismount_all_passengers(caller);
                        if self.vehicle.get_hurt_time() == 0 {
                            self.vehicle.set_hurt_dir(-self.vehicle.get_hurt_dir());
                            self.vehicle.set_hurt_time(10);
                            self.vehicle.set_damage(50.0);
                            self.vehicle.send_wobble_metadata();
                        }
                    }
                    _ => {}
                }
            }
        }

        if let MinecartKind::Tnt(minecart) = &self.kind
            && minecart.tick(&self.vehicle.entity)
        {
            return;
        }

        let mut velocity = self.vehicle.entity.velocity.load();

        let mut has_driver = false;
        let mut driver_input = 0;
        let mut driver_yaw = 0.0f32;

        {
            let passengers = self
                .vehicle
                .entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(passenger) = passengers.first()
                && let Some(player) = passenger.get_player()
            {
                driver_input = player.last_input.load(Ordering::Relaxed);
                driver_yaw = player.get_entity().yaw.load();
                has_driver = true;
            }
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

        // Vanilla post-move booster kick: shape picks a heading from a standstill.
        let mut rail_shape = None;

        if is_on_rails {
            use pumpkin_data::block_properties::RailLikeProperties;
            use pumpkin_data::block_properties::RailShapeStraight;

            let shape = if block.id == Block::RAIL.id {
                let props = RailLikeProperties::from_state_id(state_id, block);
                props.shape
            } else {
                let props = PoweredRailLikeProperties::from_state_id(state_id, block);
                match props.shape {
                    RailShapeStraight::NorthSouth => RailShape::NorthSouth,
                    RailShapeStraight::EastWest => RailShape::EastWest,
                    RailShapeStraight::AscendingEast => RailShape::AscendingEast,
                    RailShapeStraight::AscendingWest => RailShape::AscendingWest,
                    RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
                    RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
                }
            };

            rail_shape = Some(shape);

            let pos = self.vehicle.entity.pos.load();
            let block_center_bottom = Vector3::new(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y),
                f64::from(block_pos.0.z) + 0.5,
            );

            let (exit0, exit1) = get_exits(shape);
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

            // Vanilla `moveAlongTrack`: `y = pos.getY()` (rail floor). Slope: `y++`.
            target_position.y = match shape {
                RailShape::AscendingEast
                | RailShape::AscendingWest
                | RailShape::AscendingNorth
                | RailShape::AscendingSouth => f64::from(block_pos.0.y) + 1.0,
                _ => f64::from(block_pos.0.y),
            };
            // Vanilla `this.setPos(x, y, z)`: teleport onto the rail line (corner: the
            // diagonal). `set_pos` so the bounding box follows (`setPos` + `setBoundingBox`).
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

            // After the teleport: travel along the rail line.
            let mut towards_out = out_position - target_position;
            towards_out.y = 0.0;
            let towards_length = towards_out.length();
            if towards_length > 1e-5 {
                towards_out = towards_out.normalize();
                // Vanilla `moveAlongTrack`: `Math.min(2.0, movement.horizontalDistance())`.
                // Horizontal only; a slime launch would otherwise become forward speed.
                let speed = velocity.x.hypot(velocity.z).min(2.0);
                velocity = towards_out.multiply(speed, speed, speed);
            }

            // Vanilla unpowered powered rail: after re-project, before `move`. Horizontal
            // distance; `y` zeroed (`multiply(0.5, 0.0, 0.5)` or `Vec3.ZERO` below 0.03).
            if halt_track {
                if velocity.x.hypot(velocity.z) < 0.03 {
                    velocity = Vector3::new(0.0, 0.0, 0.0);
                } else {
                    velocity = Vector3::new(velocity.x * 0.5, 0.0, velocity.z * 0.5);
                }
            }

            velocity.y = 0.0;
            self.vehicle.entity.velocity.store(velocity);
        } else if !self.vehicle.entity.on_ground.load(Ordering::Relaxed) {
            velocity.y -= GRAVITY;
            self.vehicle.entity.velocity.store(velocity);
        }

        // Vanilla: `getMaxSpeed` clamp on the step handed to `move` (`moveAlongTrack` /
        // `comeOffTrack`). A slime launch is 1 block/tick; an oversized sweep uses the
        // collision context of the cell the cart is in (`rail_collision_ignore_positions`).
        let max_speed = self.max_speed();
        // Vanilla `isVehicle()`: rail step scale and post-move friction.
        let has_passengers = self.vehicle.entity.has_passengers();
        let step = if is_on_rails {
            // Vanilla `moveAlongTrack`: per-component clamp of the step, not of
            // `deltaMovement`. Occupied: 0.75. Diagonal: each axis capped, not the length.
            let scale = if has_passengers { 0.75 } else { 1.0 };
            Vector3::new(
                (scale * velocity.x).clamp(-max_speed, max_speed),
                0.0,
                (scale * velocity.z).clamp(-max_speed, max_speed),
            )
        } else {
            if velocity.x.abs() > max_speed || velocity.z.abs() > max_speed {
                // Vanilla `comeOffTrack`: clamp X/Z separately, leave `y` (gravity).
                velocity.x = velocity.x.clamp(-max_speed, max_speed);
                velocity.z = velocity.z.clamp(-max_speed, max_speed);
            }

            // Vanilla `comeOffTrack`: ground halves the delta before `move`; air drag
            // (0.95) after `move` only while airborne. Never both.
            if self.grounded_off_rail(&world, block_pos) {
                velocity = velocity.multiply(0.5, 0.5, 0.5);
            }
            self.vehicle.entity.velocity.store(velocity);
            velocity
        };

        // Skip idle ticks. Vanilla always `move`s; `power_track` still runs so a parked
        // booster can kick a standing cart.
        if step.length() > 0.001 || (is_on_rails && power_track) {
            let pre_move_pos = self.vehicle.entity.pos.load();
            self.move_entity(caller, step);

            // Vanilla `Entity.move`: zero a blocked axis, leave the rest of `deltaMovement`
            // (the step is a capped/scaled copy, not the velocity). Skip when the step
            // was not reduced, so cobweb multipliers from `move_entity` stay.
            if is_on_rails
                && ((step.x - velocity.x).abs() > 1.0e-9 || (step.z - velocity.z).abs() > 1.0e-9)
            {
                let displacement = self.vehicle.entity.pos.load() - pre_move_pos;
                let multiplier = f64::from(self.vehicle.entity.get_velocity_multiplier());
                let blocked = |requested: f64, achieved: f64| (requested - achieved).abs() > 1.0e-9;
                self.vehicle.entity.velocity.store(Vector3::new(
                    if blocked(step.x, displacement.x) {
                        0.0
                    } else {
                        velocity.x * multiplier
                    },
                    0.0,
                    if blocked(step.z, displacement.z) {
                        0.0
                    } else {
                        velocity.z * multiplier
                    },
                ));
            }

            if let MinecartKind::Tnt(minecart) = &self.kind
                && self
                    .vehicle
                    .entity
                    .horizontal_collision
                    .load(Ordering::Relaxed)
                && velocity.x.mul_add(velocity.x, velocity.z * velocity.z) >= 0.01
            {
                minecart.explode(
                    &self.vehicle.entity,
                    velocity.x.mul_add(velocity.x, velocity.z * velocity.z),
                );
                return;
            }

            // Vanilla `OldMinecartBehavior.moveAlongTrack`: re-read `getDeltaMovement()`
            // after `move` (`restituteMovementAfterCollisions`) before `applyNaturalSlowdown`.
            let velocity = self.vehicle.entity.velocity.load();

            let new_pos = self.vehicle.entity.pos.load();

            // Vanilla `OldMinecartBehavior.tick`: facing from position delta, not rail
            // shape. Compare to stored yaw so `atan2` does not flip at +/-180.
            let x_diff = pos.x - new_pos.x;
            let z_diff = pos.z - new_pos.z;
            if x_diff.mul_add(x_diff, z_diff * z_diff) > 0.001 {
                let new_yaw = wrap_degrees(z_diff.atan2(x_diff).to_degrees() as f32);
                let old_yaw = self.vehicle.entity.yaw.load();
                let rot_diff = wrap_degrees(new_yaw - old_yaw);
                let final_yaw = if (-170.0..170.0).contains(&rot_diff) {
                    new_yaw
                } else {
                    wrap_degrees(new_yaw + 180.0)
                };
                self.vehicle.entity.yaw.store(final_yaw);
            }

            {
                let passengers = self
                    .vehicle
                    .entity
                    .passengers
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for passenger in passengers.iter() {
                    passenger.get_entity().set_pos(new_pos);
                }
            }

            self.vehicle.entity.send_pos_rot();

            #[allow(clippy::useless_let_if_seq)]
            let mut friction = 0.95; // Vanilla minecart air drag

            if is_on_rails {
                friction = if has_passengers { 0.99 } else { 0.96 };
            } else if self.grounded_off_rail(&world, block_pos) {
                // Vanilla `if (!this.onGround()) scale(getAirDrag())`: ground already
                // halved the delta before `move`.
                friction = 1.0;
            }

            let mut next_vel = if is_on_rails && let MinecartKind::Furnace(minecart) = &self.kind {
                minecart.velocity(&self.vehicle.entity, velocity)
            } else if is_on_rails && let Some(inventory) = self.container() {
                container::velocity(&self.vehicle.entity, inventory, velocity)
            } else {
                velocity.multiply(friction, friction, friction)
            };
            // Vanilla `moveAlongTrack` after `move` and `applyNaturalSlowdown`.
            if is_on_rails {
                // Vanilla: if the cart crossed a cell, replace heading with the integer
                // step at current speed (`pow * (xn - pos.getX())`).
                let xn = new_pos.x.floor() as i32;
                let zn = new_pos.z.floor() as i32;
                if xn != block_pos.0.x || zn != block_pos.0.z {
                    let pow = next_vel.x.hypot(next_vel.z);
                    next_vel = Vector3::new(
                        pow * f64::from(xn - block_pos.0.x),
                        next_vel.y,
                        pow * f64::from(zn - block_pos.0.z),
                    );
                }
            }

            if power_track {
                // Vanilla `+0.06` along heading. Cap is on the step, not `deltaMovement`.
                let speed = next_vel.x.hypot(next_vel.z);
                if speed > 0.01 {
                    next_vel = Vector3::new(
                        next_vel.x + next_vel.x / speed * 0.06,
                        next_vel.y,
                        next_vel.z + next_vel.z / speed * 0.06,
                    );
                } else if let Some(shape) = rail_shape {
                    // Vanilla parked booster: nudge away from a redstone-conductor wall.
                    // Track free end, not yaw.
                    let conducts = |dx: i32, dz: i32| {
                        world
                            .get_block_state(&BlockPos(Vector3::new(
                                block_pos.0.x + dx,
                                block_pos.0.y,
                                block_pos.0.z + dz,
                            )))
                            .is_solid_block()
                    };
                    match shape {
                        RailShape::EastWest => {
                            if conducts(-1, 0) {
                                next_vel.x = 0.02;
                            } else if conducts(1, 0) {
                                next_vel.x = -0.02;
                            }
                        }
                        RailShape::NorthSouth => {
                            if conducts(0, -1) {
                                next_vel.z = 0.02;
                            } else if conducts(0, 1) {
                                next_vel.z = -0.02;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if next_vel.length() < 0.005 {
                next_vel = Vector3::new(0.0, 0.0, 0.0);
            }
            self.vehicle.entity.velocity.store(next_vel);
            if next_vel.length_squared() == 0.0 {
                self.vehicle.entity.send_velocity();
            }
        }

        // Vanilla `applyEffectsFromBlocks` every tick. At rest: once, then skip until
        // velocity is nonzero.
        let at_rest = self.vehicle.entity.velocity.load() == Vector3::new(0.0, 0.0, 0.0);
        let already_settled = self
            .block_collisions_checked_at_rest
            .swap(at_rest, Ordering::Relaxed);
        if !at_rest || !already_settled {
            self.vehicle.entity.tick_block_collisions(caller, server);
        }

        // Vanilla `OldMinecartBehavior.tick`: `pushAndPickupEntities` once, even at rest.
        self.push_entities(caller);

        if let MinecartKind::Hopper(minecart) = &self.kind {
            minecart.tick(&self.vehicle.entity);
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
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
                if !self_entity.has_passengers() && self.is_pushable() {
                    let mut vel = self_entity.velocity.load();
                    vel.x -= xa;
                    vel.z -= za;
                    self_entity.velocity.store(vel);
                    self_entity.send_velocity();
                }

                if !other_entity.has_passengers() && entity.is_pushable() {
                    let mut vel = other_entity.velocity.load();
                    vel.x += xa / 4.0;
                    vel.z += za / 4.0;
                    other_entity.velocity.store(vel);
                    other_entity.send_velocity();
                }
            }
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn requires_precise_player_collision(&self) -> bool {
        true
    }

    fn init_data_tracker(&self) {
        self.vehicle.send_wobble_metadata();
        if let MinecartKind::Furnace(minecart) = &self.kind {
            minecart.init_data_tracker(&self.vehicle.entity);
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
            && damage_type == DamageType::ARROW
            && self.vehicle.entity.fire_ticks.load(Ordering::Relaxed) > 0
        {
            let projectile_speed_squared = cause
                .map(|entity| entity.get_entity().velocity.load().length_squared())
                .unwrap_or_default();
            minecart.explode(&self.vehicle.entity, projectile_speed_squared);
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
                    container.unpack_loot();
                    let inventory: Arc<dyn Inventory> = container.clone();
                    world.scatter_inventory(&position, &inventory);
                }
                if let Some(item) = self.drop_item() {
                    world.drop_stack(&position, ItemStack::new(1, item));
                }
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                let custom_name = self.vehicle.entity.custom_name.load().as_ref().clone();
                minecart.interact(custom_name, player);
                true
            }
            MinecartKind::Furnace(minecart) => {
                minecart.interact(&self.vehicle.entity, player, item_stack)
            }
            MinecartKind::Hopper(minecart) => {
                let custom_name = self.vehicle.entity.custom_name.load().as_ref().clone();
                minecart.interact(custom_name, player);
                true
            }
            MinecartKind::Rideable(_) => RideableMinecart::interact(&self.vehicle.entity, player),
            MinecartKind::Tnt(_) | MinecartKind::Other => false,
        }
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        if self.vehicle.entity.has_passenger(player.entity_id()) {
            return;
        }

        if player.is_spectator() {
            return;
        }

        let player_pos = player.get_entity().pos.load();
        let minecart_pos = self.vehicle.entity.pos.load();

        let mut diff_x = minecart_pos.x - player_pos.x;
        let mut diff_z = minecart_pos.z - player_pos.z;

        let dist_sq = diff_x * diff_x + diff_z * diff_z;
        if dist_sq > 0.0001 {
            let dist = dist_sq.sqrt();
            diff_x /= dist;
            diff_z /= dist;

            let push_force = 0.1;
            let mut vel = self.vehicle.entity.velocity.load();
            vel.x += diff_x * push_force;
            vel.z += diff_z * push_force;

            let horizontal_speed = vel.x.hypot(vel.z);
            if horizontal_speed > 0.4 {
                vel.x = (vel.x / horizontal_speed) * 0.4;
                vel.z = (vel.z / horizontal_speed) * 0.4;
            }

            self.vehicle.entity.velocity.store(vel);
            self.vehicle.entity.send_velocity();
        }
    }

    // No `move_entity` override: vanilla `AbstractMinecart.move` extra body is
    // `useExperimentalMovement` (`minecart_improvements`, off). Default `super.move`.
    // `push_entities` runs at the end of `tick` (`OldMinecartBehavior`).

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
