use std::sync::{PoisonError, atomic::Ordering::Relaxed};

use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        PoweredRailLikeProperties, RailLikeProperties, RailShape, RailShapeStraight,
    },
    tag::{self, Taggable},
};
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_util::math::{atan2, cos, position::BlockPos, sin, vector3::Vector3, wrap_degrees};

use super::{MinecartEntity, MinecartKind, container};
use crate::{entity::EntityBase, world::World};

const GRAVITY: f64 = 0.04;
const WATER_GRAVITY: f64 = 0.005;
pub(super) const AIR_DRAG: f64 = 0.95f32 as f64;
const RAIL_HEIGHT_OFFSET: f64 = 0.0625;
const ROTATION_MOVEMENT_THRESHOLD: f64 = 0.001;
const FLIP_ROTATION_THRESHOLD: f32 = 170.0;
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
const PLAYER_START_MAX_SPEED_SQUARED: f64 = 0.01;
const PLAYER_START_ACCELERATION: f64 = 0.001;
const SLOPE_HEIGHT_SPEED_FACTOR: f64 = 0.05;

pub(super) struct Rail {
    position: BlockPos,
    block: &'static Block,
    state: BlockStateId,
}

impl Rail {
    pub(super) fn at(world: &World, position: Vector3<f64>) -> Option<Self> {
        let position = BlockPos::floored_v(position);
        let below = position.down();
        let position = if world
            .get_block(&below)
            .has_tag(&tag::Block::MINECRAFT_RAILS)
        {
            below
        } else {
            position
        };
        let (block, state) = world.get_block_and_state_id(&position);
        block.has_tag(&tag::Block::MINECRAFT_RAILS).then_some(Self {
            position,
            block,
            state,
        })
    }

    pub(super) fn activator_power(&self) -> Option<bool> {
        (self.block == &Block::ACTIVATOR_RAIL)
            .then(|| PoweredRailLikeProperties::from_state_id(self.state).powered)
    }

    fn powered_rail(&self) -> Option<PoweredRailLikeProperties> {
        (self.block == &Block::POWERED_RAIL)
            .then(|| PoweredRailLikeProperties::from_state_id(self.state))
    }

    fn shape(&self) -> RailShape {
        if self.block == &Block::RAIL {
            RailLikeProperties::from_state_id(self.state).shape
        } else {
            match PoweredRailLikeProperties::from_state_id(self.state).shape {
                RailShapeStraight::NorthSouth => RailShape::NorthSouth,
                RailShapeStraight::EastWest => RailShape::EastWest,
                RailShapeStraight::AscendingEast => RailShape::AscendingEast,
                RailShapeStraight::AscendingWest => RailShape::AscendingWest,
                RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
                RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
            }
        }
    }

    fn center(&self) -> Vector3<f64> {
        self.position.0.to_f64().add_raw(0.5, 0.0, 0.5)
    }

    fn height_at(&self, position: Vector3<f64>) -> f64 {
        let (first, second) = exits(self.shape());
        let center = self.center();
        if first.y == second.y {
            return center.y + RAIL_HEIGHT_OFFSET;
        }

        // Each endpoint is one block from the center and the lower one is one block down.
        let across = (second - first).multiply(0.5, 0.0, 0.5);
        let start = center + first * 0.5;
        let progress = (position - start).dot(&across);
        let rise = second.y - first.y;
        let slope_offset = if rise < 0.0 { 1.0 } else { 0.5 };
        start.y + RAIL_HEIGHT_OFFSET + rise * progress + slope_offset
    }
}

const fn exits(shape: RailShape) -> (Vector3<f64>, Vector3<f64>) {
    match shape {
        RailShape::NorthSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::EastWest => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::AscendingEast => (Vector3::new(-1.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::AscendingWest => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, -1.0, 0.0)),
        RailShape::AscendingNorth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, -1.0, 1.0)),
        RailShape::AscendingSouth => (Vector3::new(0.0, -1.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::SouthEast => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::SouthWest => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthWest => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthEast => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(1.0, 0.0, 0.0)),
    }
}

impl MinecartEntity {
    pub(super) fn update_classic_rotation(
        &self,
        previous_position: Vector3<f64>,
        previous_yaw: f32,
    ) {
        let entity = self.get_entity();
        let movement = previous_position - entity.pos.load();
        let flipped = self.flipped_rotation.load(Relaxed);
        let mut yaw = entity.yaw.load();
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
            self.flipped_rotation.store(!flipped, Relaxed);
        }
        entity.set_rotation(yaw % 360.0, 0.0);
    }

    fn max_speed(&self, in_water: bool) -> f64 {
        let speed = if in_water {
            MAX_RAIL_SPEED_IN_WATER
        } else {
            MAX_RAIL_SPEED
        };
        if matches!(self.kind, MinecartKind::Furnace(_)) {
            speed * if in_water { 0.75 } else { 0.5 }
        } else {
            speed
        }
    }

    pub(super) fn classic_gravity(&self) -> f64 {
        let entity = self.get_entity();
        if entity.has_no_gravity() {
            0.0
        } else if entity.touching_water.load(Relaxed) {
            WATER_GRAVITY
        } else {
            GRAVITY
        }
    }

    pub(super) fn apply_classic_gravity(&self) {
        let entity = self.get_entity();
        entity.velocity.store(
            entity
                .velocity
                .load()
                .add_raw(0.0, -self.classic_gravity(), 0.0),
        );
    }

    pub(super) fn come_off_track(&self, caller: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        let max_speed = self.max_speed(entity.touching_water.load(Relaxed));
        let mut velocity = entity.velocity.load();
        // An off-track cart caps stored velocity as well as the current displacement.
        velocity.x = velocity.x.clamp(-max_speed, max_speed);
        velocity.z = velocity.z.clamp(-max_speed, max_speed);
        if entity.on_ground.load(Relaxed) {
            velocity = velocity * 0.5;
        }
        entity.velocity.store(velocity);
        if !self.move_with_destination_effects(caller, velocity) {
            return false;
        }
        if !entity.on_ground.load(Relaxed) {
            entity.velocity.store(entity.velocity.load() * AIR_DRAG);
        }
        true
    }

    pub(super) fn move_along_classic_rail(&self, caller: &dyn EntityBase, rail: &Rail) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let position = entity.pos.load();
        let in_water = entity.touching_water.load(Relaxed);
        let powered = rail.powered_rail();
        let (first, second) = exits(rail.shape());
        let direction = (second - first).multiply(1.0, 0.0, 1.0);
        let ascending = first.y != second.y;
        let mut velocity = entity.velocity.load();
        if ascending {
            let downhill = if first.y < second.y { first } else { second };
            let slope = if in_water {
                SLOPE_ACCELERATION * IN_WATER_SLOPE_FACTOR
            } else {
                SLOPE_ACCELERATION
            };
            velocity += downhill.multiply(slope, 0.0, slope);
        }
        velocity = align_velocity(velocity, direction);

        let brake = powered.is_some_and(|rail| !rail.powered);
        velocity = apply_rider_start(velocity, self.rider_intent(), brake);
        entity.velocity.store(velocity);

        let center = rail.center();
        let start = center + first * 0.5;
        let axis = direction * 0.5;
        let progress = if first.x == second.x {
            position.z - f64::from(rail.position.0.z)
        } else if first.z == second.z {
            position.x - f64::from(rail.position.0.x)
        } else {
            (position - start).dot(&axis) * 2.0
        };
        let adjusted_y = if ascending { center.y + 1.0 } else { center.y };
        let track_position = start + axis * progress;
        entity.set_pos(Vector3::new(track_position.x, adjusted_y, track_position.z));

        let occupied = entity.has_passengers();
        let scale = if occupied {
            OCCUPIED_RAIL_MOTION_FACTOR
        } else {
            1.0
        };
        let max_speed = self.max_speed(in_water);
        let displacement = Vector3::new(
            (velocity.x * scale).clamp(-max_speed, max_speed),
            0.0,
            (velocity.z * scale).clamp(-max_speed, max_speed),
        );
        if !self.move_with_destination_effects(caller, displacement) {
            return false;
        }

        let mut end = entity.pos.load();
        let block_change = BlockPos::floored_v(end).0 - rail.position.0;
        let delta = block_change.to_f64();
        for exit in [first, second] {
            if exit.y != 0.0 && delta.x == exit.x && delta.z == exit.z {
                end.y += exit.y;
                entity.set_pos(end);
                break;
            }
        }

        let mut velocity =
            self.classic_slowdown(entity.velocity.load(), in_water, entity.has_passengers());
        if let Some(next) = Rail::at(&world, end) {
            let height = next.height_at(end);
            let speed = velocity.horizontal_length();
            if speed > 0.0 {
                let height_change = (rail.height_at(position) - height) * SLOPE_HEIGHT_SPEED_FACTOR;
                let scale = (speed + height_change) / speed;
                velocity = velocity.multiply(scale, 1.0, scale);
            }
            entity.set_pos(Vector3::new(end.x, height, end.z));
        }

        if block_change.x != 0 || block_change.z != 0 {
            let speed = velocity.horizontal_length();
            velocity = Vector3::new(speed * delta.x, velocity.y, speed * delta.z);
        }
        if let Some(powered) = powered.filter(|rail| rail.powered) {
            velocity = powered_velocity(&world, &rail.position, powered.shape, velocity);
        }
        entity.velocity.store(velocity);
        true
    }

    fn rider_intent(&self) -> Vector3<f64> {
        let rider = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .first()
            .cloned();
        rider
            .as_deref()
            .and_then(EntityBase::get_player)
            .map_or(Vector3::default(), |player| {
                player_move_intent(
                    player.last_input.load(Relaxed),
                    player.get_entity().yaw.load(),
                )
            })
    }

    fn classic_slowdown(
        &self,
        mut velocity: Vector3<f64>,
        in_water: bool,
        occupied: bool,
    ) -> Vector3<f64> {
        if let Some(inventory) = self.container() {
            return container::velocity(inventory, velocity, in_water);
        }
        if let MinecartKind::Furnace(furnace) = &self.kind {
            velocity = furnace.velocity(velocity, in_water);
        }
        let drag = if occupied { 0.997 } else { 0.96 };
        let movement = velocity.multiply(drag, 0.0, drag);
        if in_water {
            movement * AIR_DRAG
        } else {
            movement
        }
    }
}

fn align_velocity(velocity: Vector3<f64>, mut direction: Vector3<f64>) -> Vector3<f64> {
    if velocity.dot(&direction) < 0.0 {
        direction = direction * -1.0;
    }
    let length = direction.horizontal_length();
    let speed = velocity.horizontal_length().min(MAX_PROJECTED_SPEED);
    Vector3::new(
        speed * direction.x / length,
        velocity.y,
        speed * direction.z / length,
    )
}

fn player_move_intent(flags: i8, yaw: f32) -> Vector3<f64> {
    let axis = |positive, negative| {
        f64::from(i32::from(flags & positive != 0) - i32::from(flags & negative != 0))
    };
    let mut input = Vector3::new(
        axis(SPlayerInput::LEFT, SPlayerInput::RIGHT),
        0.0,
        axis(SPlayerInput::FORWARD, SPlayerInput::BACKWARD),
    );
    if input.length_squared() > 1.0 {
        input = input.normalize();
    }
    let angle = yaw.to_radians();
    let (sin, cos) = (f64::from(sin(angle)), f64::from(cos(angle)));
    Vector3::new(
        input.x * cos - input.z * sin,
        0.0,
        input.z * cos + input.x * sin,
    )
}

fn apply_rider_start(velocity: Vector3<f64>, intent: Vector3<f64>, brake: bool) -> Vector3<f64> {
    if intent.length_squared() > 0.0
        && velocity.horizontal_length_squared() < PLAYER_START_MAX_SPEED_SQUARED
    {
        return velocity + intent * PLAYER_START_ACCELERATION;
    }
    if brake {
        if velocity.horizontal_length() < UNPOWERED_RAIL_BRAKE_SPEED {
            Vector3::default()
        } else {
            velocity.multiply(0.5, 0.0, 0.5)
        }
    } else {
        velocity
    }
}

fn powered_velocity(
    world: &World,
    position: &BlockPos,
    shape: RailShapeStraight,
    mut velocity: Vector3<f64>,
) -> Vector3<f64> {
    let speed = velocity.horizontal_length();
    if speed > POWERED_RAIL_MIN_SPEED {
        velocity.x += velocity.x / speed * POWERED_RAIL_BOOST;
        velocity.z += velocity.z / speed * POWERED_RAIL_BOOST;
        return velocity;
    }
    let conducts = |neighbor| world.get_block_state(&neighbor).is_solid_block();
    match shape {
        RailShapeStraight::EastWest if conducts(position.west()) => {
            velocity.x = POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::EastWest if conducts(position.east()) => {
            velocity.x = -POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::NorthSouth if conducts(position.north()) => {
            velocity.z = POWERED_RAIL_START_SPEED;
        }
        RailShapeStraight::NorthSouth if conducts(position.south()) => {
            velocity.z = -POWERED_RAIL_START_SPEED;
        }
        _ => {}
    }
    velocity
}
