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
use pumpkin_data::block_properties::{PoweredRailLikeProperties, RailShape};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

use crate::entity::vehicle::vehicle::VehicleEntity;
use crate::entity::velocity;
use chest::ChestMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use rideable::RideableMinecart;
use tnt::TntMinecart;

/// Vanilla `AbstractMinecart.exits`: a slope's lower end sits one block down.
const fn get_exits(
    shape: pumpkin_data::block_properties::RailShape,
) -> (Vector3<f64>, Vector3<f64>) {
    use pumpkin_data::block_properties::RailShape;
    const WEST: Vector3<f64> = Vector3::new(-1.0, 0.0, 0.0);
    const EAST: Vector3<f64> = Vector3::new(1.0, 0.0, 0.0);
    const NORTH: Vector3<f64> = Vector3::new(0.0, 0.0, -1.0);
    const SOUTH: Vector3<f64> = Vector3::new(0.0, 0.0, 1.0);
    const fn below(exit: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(exit.x, -1.0, exit.z)
    }
    match shape {
        RailShape::NorthSouth => (NORTH, SOUTH),
        RailShape::EastWest => (WEST, EAST),
        RailShape::AscendingEast => (below(WEST), EAST),
        RailShape::AscendingWest => (WEST, below(EAST)),
        RailShape::AscendingNorth => (NORTH, below(SOUTH)),
        RailShape::AscendingSouth => (below(NORTH), SOUTH),
        RailShape::SouthEast => (SOUTH, EAST),
        RailShape::SouthWest => (SOUTH, WEST),
        RailShape::NorthWest => (NORTH, WEST),
        RailShape::NorthEast => (NORTH, EAST),
    }
}

/// Vanilla `getCurrentBlockPosOrRailBelow`.
fn rail_block_pos(world: &World, pos: Vector3<f64>) -> BlockPos {
    let block_pos = BlockPos::floored_v(pos);
    let below = BlockPos(Vector3::new(
        block_pos.0.x,
        block_pos.0.y - 1,
        block_pos.0.z,
    ));
    if world
        .get_block(&below)
        .has_tag(&tag::Block::MINECRAFT_RAILS)
    {
        below
    } else {
        block_pos
    }
}

fn rail_shape_of(block: &Block, state_id: BlockStateId) -> Option<RailShape> {
    use pumpkin_data::block_properties::{BlockProperties, RailLikeProperties, RailShapeStraight};
    if block.id == Block::RAIL.id {
        return Some(RailLikeProperties::from_state_id(state_id).shape);
    }
    if !PoweredRailLikeProperties::handles_block_id(block.id) {
        return None;
    }
    Some(
        match PoweredRailLikeProperties::from_state_id(state_id).shape {
            RailShapeStraight::NorthSouth => RailShape::NorthSouth,
            RailShapeStraight::EastWest => RailShape::EastWest,
            RailShapeStraight::AscendingEast => RailShape::AscendingEast,
            RailShapeStraight::AscendingWest => RailShape::AscendingWest,
            RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
            RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
        },
    )
}

/// How far along the rail line (`x0`/`z0` + `xd`/`zd`) the point sits.
fn rail_progress(
    point: Vector3<f64>,
    block_x: f64,
    block_z: f64,
    x0: f64,
    z0: f64,
    xd: f64,
    zd: f64,
) -> f64 {
    if xd == 0.0 {
        point.z - block_z
    } else if zd == 0.0 {
        point.x - block_x
    } else {
        ((point.x - x0) * xd + (point.z - z0) * zd) * 2.0
    }
}

/// Vanilla `OldMinecartBehavior.getPos`: the point on the rail under `pos`.
fn rail_pos(world: &World, pos: Vector3<f64>) -> Option<Vector3<f64>> {
    let block_pos = rail_block_pos(world, pos);
    let (block, state_id) = world.get_block_and_state_id(&block_pos);
    let (exit0, exit1) = get_exits(rail_shape_of(block, state_id)?);
    let (block_x, block_y, block_z) = (
        f64::from(block_pos.0.x),
        f64::from(block_pos.0.y),
        f64::from(block_pos.0.z),
    );
    let x0 = block_x + 0.5 + exit0.x * 0.5;
    let y0 = block_y + RAIL_HEIGHT_OFFSET + exit0.y * 0.5;
    let z0 = block_z + 0.5 + exit0.z * 0.5;
    let xd = block_x + 0.5 + exit1.x * 0.5 - x0;
    let yd = (block_y + RAIL_HEIGHT_OFFSET + exit1.y * 0.5 - y0) * 2.0;
    let zd = block_z + 0.5 + exit1.z * 0.5 - z0;
    let progress = rail_progress(pos, block_x, block_z, x0, z0, xd, zd);
    let lift = if yd < 0.0 {
        1.0
    } else if yd > 0.0 {
        0.5
    } else {
        0.0
    };
    Some(Vector3::new(
        x0 + xd * progress,
        y0 + yd * progress + lift,
        z0 + zd * progress,
    ))
}

/// powered rail boost, after `applyNaturalSlowdown`.
fn boost(
    world: &World,
    pos: BlockPos,
    shape: RailShape,
    mut movement: Vector3<f64>,
) -> Vector3<f64> {
    let speed = movement.horizontal_length();
    if speed > 0.01 {
        movement.x += movement.x / speed * 0.06;
        movement.z += movement.z / speed * 0.06;
        return movement;
    }
    // A standing cart starts away from a redstone conductor.
    let conductor = |x: i32, z: i32| {
        world
            .get_block_state(&BlockPos(Vector3::new(pos.0.x + x, pos.0.y, pos.0.z + z)))
            .is_solid_block()
    };
    match shape {
        RailShape::EastWest if conductor(-1, 0) => movement.x = 0.02,
        RailShape::EastWest if conductor(1, 0) => movement.x = -0.02,
        RailShape::NorthSouth if conductor(0, -1) => movement.z = 0.02,
        RailShape::NorthSouth if conductor(0, 1) => movement.z = -0.02,
        _ => {}
    }
    movement
}

const GRAVITY: f64 = 0.04;
const RAIL_HEIGHT_OFFSET: f64 = 0.0625;
const AIR_DRAG: f32 = 0.95;
/// Pumpkin: vanilla never zeroes a sliding cart. Below this the rest of the slide
/// is under 0.04 blocks, even with the 0.997 rider slowdown.
const STOP_SPEED: f64 = 1.0E-4;

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
    flipped: AtomicBool,
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
            flipped: AtomicBool::new(false),
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match &self.kind {
            MinecartKind::Chest(minecart) => Some(minecart.inventory()),
            MinecartKind::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }

    /// Vanilla `getMaxSpeed` (old behaviour): the per-axis cap of one step.
    fn max_speed(&self) -> f64 {
        let in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed);
        let base = if in_water { 0.2 } else { 0.4 };
        match self.kind {
            MinecartKind::Furnace(_) if in_water => base * 0.75,
            MinecartKind::Furnace(_) => base * 0.5,
            _ => base,
        }
    }

    /// Vanilla `ServerPlayer.getLastClientMoveIntent` of a player in the front seat.
    fn rider_move_intent(&self) -> Vector3<f64> {
        let passengers = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(player) = passengers
            .first()
            .and_then(|passenger| passenger.get_player())
        else {
            return Vector3::default();
        };
        let input = player.last_input.load(Ordering::Relaxed);
        let axis = |positive: i8, negative: i8| match (input & positive != 0, input & negative != 0)
        {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };
        let intent = Vector3::new(
            axis(SPlayerInput::LEFT, SPlayerInput::RIGHT),
            0.0,
            axis(SPlayerInput::FORWARD, SPlayerInput::BACKWARD),
        );
        // Vanilla `Entity.getInputVector` with speed 1.
        let length = intent.length_squared();
        if length < 1.0E-7 {
            return Vector3::default();
        }
        let intent = if length > 1.0 {
            intent.normalize()
        } else {
            intent
        };
        let yaw = player.get_entity().yaw.load().to_radians();
        let (sin, cos) = (f64::from(yaw.sin()), f64::from(yaw.cos()));
        Vector3::new(
            intent.x * cos - intent.z * sin,
            0.0,
            intent.z * cos + intent.x * sin,
        )
    }

    /// Vanilla `OldMinecartBehavior.moveAlongTrack`.
    fn move_along_track(
        &self,
        caller: &dyn EntityBase,
        world: &World,
        pos: BlockPos,
        shape: RailShape,
        power_track: bool,
        mut halt_track: bool,
    ) {
        let entity = &self.vehicle.entity;
        let start = entity.pos.load();
        let old_rail_pos = rail_pos(world, start);
        let in_water = entity.touching_water.load(Ordering::Relaxed);
        let mut movement = entity.velocity.load();

        let slide = if in_water { 0.0078125 * 0.2 } else { 0.0078125 };
        match shape {
            RailShape::AscendingEast => movement.x -= slide,
            RailShape::AscendingWest => movement.x += slide,
            RailShape::AscendingNorth => movement.z += slide,
            RailShape::AscendingSouth => movement.z -= slide,
            _ => {}
        }
        let y = f64::from(pos.0.y) + if shape.is_ascending() { 1.0 } else { 0.0 };

        // Redirect along the rail.
        let (exit0, exit1) = get_exits(shape);
        let (mut xd, mut zd) = (exit1.x - exit0.x, exit1.z - exit0.z);
        let length = xd.hypot(zd);
        if movement.x * xd + movement.z * zd < 0.0 {
            (xd, zd) = (-xd, -zd);
        }
        let pow = movement.horizontal_length().min(2.0);
        movement = Vector3::new(pow * xd / length, movement.y, pow * zd / length);

        // A rider only nudges a (nearly) standing cart.
        let move_intent = self.rider_move_intent();
        if move_intent.length_squared() > 0.0 && movement.horizontal_length_squared() < 0.01 {
            movement.x += move_intent.x * 0.001;
            movement.z += move_intent.z * 0.001;
            halt_track = false;
        }
        if halt_track {
            movement = if movement.horizontal_length() < 0.03 {
                Vector3::default()
            } else {
                movement.multiply(0.5, 0.0, 0.5)
            };
        }

        // Snap onto the rail line.
        let (block_x, block_z) = (f64::from(pos.0.x), f64::from(pos.0.z));
        let x0 = block_x + 0.5 + exit0.x * 0.5;
        let z0 = block_z + 0.5 + exit0.z * 0.5;
        let (xd, zd) = (
            block_x + 0.5 + exit1.x * 0.5 - x0,
            block_z + 0.5 + exit1.z * 0.5 - z0,
        );
        let progress = rail_progress(start, block_x, block_z, x0, z0, xd, zd);
        entity.set_pos(Vector3::new(x0 + xd * progress, y, z0 + zd * progress));

        // The step is capped, `deltaMovement` is not: an overspeed cart keeps its speed.
        let scale = if entity.has_passengers() { 0.75 } else { 1.0 };
        let max_speed = self.max_speed();
        let step = Vector3::new(
            (scale * movement.x).clamp(-max_speed, max_speed),
            0.0,
            (scale * movement.z).clamp(-max_speed, max_speed),
        );
        if step.length_squared() > 0.0 {
            self.move_entity(caller, step);
        }

        // Climb onto the higher end of a slope.
        let moved = entity.pos.load();
        let cell = |exit: Vector3<f64>| {
            exit.y != 0.0
                && moved.x.floor() as i32 - pos.0.x == exit.x as i32
                && moved.z.floor() as i32 - pos.0.z == exit.z as i32
        };
        if let Some(exit) = [exit0, exit1].into_iter().find(|exit| cell(*exit)) {
            entity.set_pos(Vector3::new(moved.x, moved.y + exit.y, moved.z));
        }

        movement = self.apply_natural_slowdown(movement);

        // Height lost on the rail becomes speed.
        let moved = entity.pos.load();
        if let (Some(old), Some(new)) = (old_rail_pos, rail_pos(world, moved)) {
            let speed = (old.y - new.y) * 0.05;
            let other_pow = movement.horizontal_length();
            if other_pow > 0.0 {
                let factor = (other_pow + speed) / other_pow;
                movement = movement.multiply(factor, 1.0, factor);
            }
            entity.set_pos(Vector3::new(moved.x, new.y, moved.z));
        }

        // Entering the next cell: head straight into it.
        let moved = entity.pos.load();
        let (xn, zn) = (moved.x.floor() as i32, moved.z.floor() as i32);
        if xn != pos.0.x || zn != pos.0.z {
            let other_pow = movement.horizontal_length();
            movement = Vector3::new(
                other_pow * f64::from(xn - pos.0.x),
                movement.y,
                other_pow * f64::from(zn - pos.0.z),
            );
        }

        if power_track {
            movement = boost(world, pos, shape, movement);
        }
        self.store_movement(movement);
    }

    /// Vanilla `AbstractMinecart.comeOffTrack`.
    fn come_off_track(&self, caller: &dyn EntityBase, world: &World, pos: BlockPos) {
        let entity = &self.vehicle.entity;
        let mut movement = entity.velocity.load();
        if !entity.on_ground.load(Ordering::Relaxed) {
            movement.y -= GRAVITY;
        }
        let max_speed = self.max_speed();
        movement.x = movement.x.clamp(-max_speed, max_speed);
        movement.z = movement.z.clamp(-max_speed, max_speed);
        if self.grounded(world, pos) {
            movement = movement * 0.5;
        }
        if movement.length_squared() > 0.0 {
            self.move_entity(caller, movement);
            // Pumpkin `move_entity` leaves the collided step as velocity.
            movement = entity.velocity.load();
        }
        if !self.grounded(world, pos) {
            movement = movement * f64::from(AIR_DRAG);
        }
        self.store_movement(movement);
    }

    /// `onGround`, or resting on a block the flag missed.
    fn grounded(&self, world: &World, pos: BlockPos) -> bool {
        let below = world.get_block(&BlockPos(Vector3::new(pos.0.x, pos.0.y - 1, pos.0.z)));
        self.vehicle.entity.on_ground.load(Ordering::Relaxed)
            || (below.id != Block::AIR.id
                && below.id != Block::WATER.id
                && below.id != Block::LAVA.id)
    }

    /// Vanilla `applyNaturalSlowdown` of each minecart kind. Drops vertical speed.
    fn apply_natural_slowdown(&self, movement: Vector3<f64>) -> Vector3<f64> {
        let entity = &self.vehicle.entity;
        match &self.kind {
            MinecartKind::Furnace(minecart) => minecart.velocity(entity, movement),
            _ if let Some(inventory) = self.container() => {
                container::velocity(entity, inventory, movement)
            }
            _ => {
                // Vanilla `getSlowdownFactor`
                let mut factor = if entity.has_passengers() { 0.997 } else { 0.96 };
                if entity.touching_water.load(Ordering::Relaxed) {
                    factor *= f64::from(0.95f32);
                }
                movement.multiply(factor, 0.0, factor)
            }
        }
    }

    fn store_movement(&self, mut movement: Vector3<f64>) {
        if movement.length() < STOP_SPEED {
            movement = Vector3::default();
        }
        self.vehicle.entity.velocity.store(movement);
    }

    /// face the movement, flip on reversal
    fn update_rotation(&self, start: Vector3<f64>, start_yaw: f32) {
        let entity = &self.vehicle.entity;
        let pos = entity.pos.load();
        let (x_diff, z_diff) = (start.x - pos.x, start.z - pos.z);
        let mut yaw = entity.yaw.load();
        if x_diff * x_diff + z_diff * z_diff > 0.001 {
            yaw = z_diff.atan2(x_diff).to_degrees() as f32;
            if self.flipped.load(Ordering::Relaxed) {
                yaw += 180.0;
            }
        }
        let rot_diff = wrap_degrees(yaw - start_yaw);
        if !(-170.0..170.0).contains(&rot_diff) {
            yaw += 180.0;
            self.flipped.fetch_xor(true, Ordering::Relaxed);
        }
        entity.pitch.store(0.0);
        entity.yaw.store(yaw % 360.0);
    }

    /// Vanilla `OldMinecartBehavior.pushAndPickupEntities`.
    fn push_and_pickup_entities(&self, caller: &dyn EntityBase) {
        let entity = &self.vehicle.entity;
        let world = entity.world.load();
        let hitbox = entity.bounding_box.load().expand(0.2, 0.0, 0.2);
        let is_minecart = |other: &dyn EntityBase| other.cast_any().is::<Self>();

        if !(matches!(self.kind, MinecartKind::Rideable(_))
            && entity.velocity.load().horizontal_length_squared() >= 0.01)
        {
            for other in world.get_all_at_box(&hitbox) {
                if other.get_entity().entity_id != entity.entity_id
                    && !entity.has_passenger(other.get_entity().entity_id)
                    && other.is_pushable()
                    && is_minecart(other.as_ref())
                {
                    other.push(caller);
                }
            }
            return;
        }

        let pushable = velocity::pushable_entities(caller, &hitbox);
        for other in &pushable {
            let other_entity = other.get_entity();
            if other.get_player().is_none()
                && other_entity.entity_type.id != EntityType::IRON_GOLEM.id
                && !is_minecart(other.as_ref())
                && !entity.has_passengers()
                && !other.is_passenger()
            {
                // `startRiding` fails on boarding cooldown, and then nothing happens.
                if other_entity.riding_cooldown.load(Ordering::Relaxed) == 0
                    && self.vehicle.collide_entity(other_entity.entity_id)
                    && let Some(this) = world.get_entity_by_id(entity.entity_id)
                {
                    entity.add_passenger(this, other.clone());
                }
            } else if is_minecart(other.as_ref())
                || self.vehicle.collide_entity(other_entity.entity_id)
            {
                // A minecart fires the collision event in its own `push`.
                other.push(caller);
            }
        }
    }

    /// Vanilla `AbstractMinecart.pushOtherMinecart`, old movement.
    fn push_other_minecart(&self, other: &Self, xa: f64, za: f64) {
        let this = &self.vehicle.entity;
        let that = &other.vehicle.entity;
        let (pos, other_pos) = (this.pos.load(), that.pos.load());
        let dir = Vector3::new(other_pos.x - pos.x, 0.0, other_pos.z - pos.z).normalize();
        let yaw = this.yaw.load().to_radians();
        let facing = Vector3::new(f64::from(yaw.cos()), 0.0, f64::from(yaw.sin())).normalize();
        if dir.dot(&facing).abs() < f64::from(0.8f32) {
            return;
        }

        let movement = this.velocity.load();
        let other_movement = that.velocity.load();
        let slow_down = |entity: &Entity, velocity: Vector3<f64>, factor: f64| {
            entity
                .velocity
                .store(velocity.multiply(factor, 1.0, factor));
        };
        match (
            matches!(self.kind, MinecartKind::Furnace(_)),
            matches!(other.kind, MinecartKind::Furnace(_)),
        ) {
            (false, true) => {
                slow_down(this, movement, 0.2);
                this.push_velocity(other_movement.x - xa, other_movement.z - za);
                slow_down(that, other_movement, 0.95);
            }
            (true, false) => {
                slow_down(that, other_movement, 0.2);
                that.push_velocity(movement.x + xa, movement.z + za);
                slow_down(this, movement, 0.95);
            }
            _ => {
                #[expect(clippy::manual_midpoint, reason = "vanilla rounding")]
                let (xdd, zdd) = (
                    (other_movement.x + movement.x) / 2.0,
                    (other_movement.z + movement.z) / 2.0,
                );
                slow_down(this, movement, 0.2);
                this.push_velocity(xdd - xa, zdd - za);
                slow_down(that, other_movement, 0.2);
                that.push_velocity(xdd + xa, zdd + za);
            }
        }
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
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.vehicle.tick();
        if let MinecartKind::Furnace(minecart) = &self.kind {
            minecart.tick(&self.vehicle.entity);
        }

        let world = self.vehicle.entity.world.load();
        let start = self.vehicle.entity.pos.load();
        let start_yaw = self.vehicle.entity.yaw.load();
        let block_pos = rail_block_pos(&world, self.vehicle.entity.pos.load());
        let (block, state_id) = world.get_block_and_state_id(&block_pos);
        let is_powered_rail = block.id == Block::POWERED_RAIL.id;
        let is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
        let rail_shape = rail_shape_of(block, state_id);

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

        let mut power_track = false;
        let mut halt_track = false;
        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id);
            let powered = props.powered;

            if is_powered_rail {
                // Vanilla `moveAlongTrack`: brake before the move, boost after it.
                power_track = powered;
                halt_track = !powered;
            } else if powered {
                match &self.kind {
                    MinecartKind::Tnt(minecart) => {
                        minecart.prime(&self.vehicle.entity, 80);
                    }
                    MinecartKind::Rideable(_) => {
                        if let Ok(passengers) = self.vehicle.entity.passengers.try_lock() {
                            let p_ids: Vec<i32> = passengers
                                .iter()
                                .map(|p| p.get_entity().entity_id)
                                .collect();
                            if !p_ids.is_empty() {
                                let world = self.vehicle.entity.world.load();
                                let vid = self.vehicle.entity.entity_id;
                                if let Some(v) = world.get_entity_by_id(vid) {
                                    for pid in p_ids {
                                        v.get_entity().remove_passenger_sync(pid);
                                    }
                                }
                            }
                        }
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

        match rail_shape {
            Some(shape) => {
                self.move_along_track(caller, &world, block_pos, shape, power_track, halt_track);
            }
            None => self.come_off_track(caller, &world, block_pos),
        }

        {
            let pos = self.vehicle.entity.pos.load();
            let passengers = self
                .vehicle
                .entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for passenger in passengers.iter() {
                passenger.get_entity().set_pos(pos);
            }
        }

        self.update_rotation(start, start_yaw);
        self.push_and_pickup_entities(caller);

        // Vanilla `MinecartTNT.tick`, after the pushes.
        let velocity = self.vehicle.entity.velocity.load();
        if let MinecartKind::Tnt(minecart) = &self.kind
            && self
                .vehicle
                .entity
                .horizontal_collision
                .load(Ordering::Relaxed)
            && velocity.horizontal_length_squared() >= 0.01
        {
            minecart.explode(&self.vehicle.entity, velocity.horizontal_length_squared());
            return;
        }

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

    /// Bedrock minecart origin sits 0.35 above the Java one.
    fn bedrock_y_offset(&self) -> f64 {
        0.35
    }

    /// Vanilla `AbstractMinecart.push`.
    fn push(&self, entity: &dyn EntityBase) {
        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
            || self_entity.has_passenger(other_entity.entity_id)
        {
            return;
        }

        let mut xa = other_entity.pos.load().x - self_entity.pos.load().x;
        let mut za = other_entity.pos.load().z - self_entity.pos.load().z;
        let mut dd = xa * xa + za * za;
        // Plugin hook: only for pushes that happen.
        if dd < f64::from(1.0E-4f32) || !self.vehicle.collide_entity(other_entity.entity_id) {
            return;
        }
        dd = dd.sqrt();
        xa /= dd;
        za /= dd;
        let pow = (1.0 / dd).min(1.0);
        xa = xa * pow * f64::from(0.1f32) * 0.5;
        za = za * pow * f64::from(0.1f32) * 0.5;

        if let Some(other) = entity.cast_any().downcast_ref::<Self>() {
            self.push_other_minecart(other, xa, za);
        } else {
            self_entity.push_velocity(-xa, -za);
            other_entity.push_velocity(xa / 4.0, za / 4.0);
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn has_entity_collisions(&self) -> bool {
        true
    }

    /// Vanilla `AbstractBoat.canVehicleCollide`.
    fn can_collide_with(&self, other: &dyn EntityBase) -> bool {
        (other.is_collidable(None) || other.is_pushable())
            && !self
                .vehicle
                .entity
                .is_passenger_of_same_vehicle(other.get_entity())
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

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
