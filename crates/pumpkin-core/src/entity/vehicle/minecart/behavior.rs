//! Vanilla `MinecartBehavior`: what the old and the experimental minecart movement share.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::block_properties::{PoweredRailLikeProperties, RailShape};
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{cos, sin};

use super::new_behavior::{self, NewMinecartBehavior};
use super::{AbstractMinecart, MinecartKind, is_minecart};
use crate::block::blocks::redstone::rails::rail_shape;
use crate::entity::living::is_allowed_by_team_rules;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use crate::world::game_event::GameEventContext;
use crate::world::scoreboard::Team;

/// Vanilla `FeatureFlags.MINECART_IMPROVEMENTS`.
const MINECART_IMPROVEMENTS: &str = "minecraft:minecart_improvements";
const IN_WATER_GRAVITY: f64 = 0.005;
const GRAVITY: f64 = 0.04;
/// Vanilla `getAirDrag`.
const AIR_DRAG: f64 = 0.95f32 as f64;
/// Vanilla `MINECART_RIDABLE_THRESHOLD`, squared horizontal speed.
pub(super) const RIDEABLE_THRESHOLD: f64 = 0.01;
/// Vanilla `1.0E-5F`, the movement epsilon of both behaviours.
pub(super) const EPSILON: f64 = 1.0e-5f32 as f64;
/// Most a rail snap or a slope exit moves a cart beyond its track speed, in either behaviour.
const RAIL_SNAP_REACH: f64 = 2.0;

/// Vanilla `AbstractMinecart.behavior`, picked once at construction like vanilla.
pub(super) enum MinecartBehavior {
    Old,
    New(NewMinecartBehavior),
}

impl MinecartBehavior {
    /// Vanilla `useExperimentalMovement`: the world's features are fixed while it runs.
    pub(super) fn for_world(world: &World) -> Self {
        let experimental = world
            .server
            .upgrade()
            .is_some_and(|server| server.is_feature_enabled(MINECART_IMPROVEMENTS));
        if experimental {
            Self::New(NewMinecartBehavior::new())
        } else {
            Self::Old
        }
    }

    pub(super) const fn new_behavior(&self) -> Option<&NewMinecartBehavior> {
        match self {
            Self::Old => None,
            Self::New(behavior) => Some(behavior),
        }
    }
}

/// Vanilla `AbstractMinecart.EXITS`: the two block offsets a rail shape connects.
pub(super) const fn exits(shape: RailShape) -> (Vector3<i32>, Vector3<i32>) {
    const fn at(x: i32, y: i32, z: i32) -> Vector3<i32> {
        Vector3::new(x, y, z)
    }
    match shape {
        RailShape::NorthSouth => (at(0, 0, -1), at(0, 0, 1)),
        RailShape::EastWest => (at(-1, 0, 0), at(1, 0, 0)),
        RailShape::AscendingEast => (at(-1, -1, 0), at(1, 0, 0)),
        RailShape::AscendingWest => (at(-1, 0, 0), at(1, -1, 0)),
        RailShape::AscendingNorth => (at(0, 0, -1), at(0, -1, 1)),
        RailShape::AscendingSouth => (at(0, -1, -1), at(0, 0, 1)),
        RailShape::SouthEast => (at(0, 0, 1), at(1, 0, 0)),
        RailShape::SouthWest => (at(0, 0, 1), at(-1, 0, 0)),
        RailShape::NorthWest => (at(0, 0, -1), at(-1, 0, 0)),
        RailShape::NorthEast => (at(0, 0, -1), at(1, 0, 0)),
    }
}

/// Rail block a cart is on.
#[derive(Clone, Copy)]
pub(super) struct Rail {
    pub pos: BlockPos,
    pub block: &'static Block,
    pub state: BlockStateId,
    pub shape: RailShape,
}

impl Rail {
    fn of(pos: BlockPos, block: &'static Block, state: BlockStateId) -> Option<Self> {
        rail_shape(block, state).map(|shape| Self {
            pos,
            block,
            state,
            shape,
        })
    }

    /// Vanilla `PoweredRailBlock.POWERED` of a powered rail, `None` for other rails.
    pub(super) fn powered_rail(&self) -> Option<bool> {
        (self.block == &Block::POWERED_RAIL)
            .then(|| PoweredRailLikeProperties::from_state_id(self.state).powered)
    }
}

/// Vanilla `ServerPlayer.getLastClientMoveIntent`.
fn move_intent(player: &Player) -> Vector3<f64> {
    let input = player.last_input.load(Ordering::Relaxed);
    let axis = |positive: i8, negative: i8| match (input & positive != 0, input & negative != 0) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    };
    let left = axis(SPlayerInput::LEFT, SPlayerInput::RIGHT);
    let forward = axis(SPlayerInput::FORWARD, SPlayerInput::BACKWARD);
    if left == 0.0 && forward == 0.0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    // `getInputVector`: a diagonal is normalized.
    let input = Vector3::new(left, 0.0, forward);
    let input = if left != 0.0 && forward != 0.0 {
        input.normalize()
    } else {
        input
    };
    let yaw = player.get_entity().yaw.load() * (std::f32::consts::PI / 180.0);
    let (sin, cos) = (f64::from(sin(yaw)), f64::from(cos(yaw)));
    Vector3::new(
        input.x * cos - input.z * sin,
        0.0,
        input.z * cos + input.x * sin,
    )
}

/// Vanilla `Vec3.normalize`: tiny vectors become zero.
pub fn normalize(vector: Vector3<f64>) -> Vector3<f64> {
    let length = vector.length();
    if length < EPSILON {
        Vector3::new(0.0, 0.0, 0.0)
    } else {
        Vector3::new(vector.x / length, vector.y / length, vector.z / length)
    }
}

/// Vanilla `EntitySelector.pushableBy`.
pub(super) fn pushable_by(own_id: i32, own_team: Option<&Team>, other: &dyn EntityBase) -> bool {
    other.get_entity().entity_id != own_id
        && !other.is_spectator()
        && other.is_pushable()
        && is_allowed_by_team_rules(own_team, other.get_team().as_ref())
}

/// The entities a rideable cart picks up instead of pushing.
pub(super) fn can_be_picked_up(other: &dyn EntityBase) -> bool {
    other.get_player().is_none()
        && other.get_entity().entity_type != &EntityType::IRON_GOLEM
        && !is_minecart(other.get_entity().entity_type)
        && !other.is_passenger()
}

impl AbstractMinecart {
    /// Vanilla `getCurrentBlockPosOrRailBelow` and the rail there.
    pub(super) fn current_rail(&self, world: &World) -> (BlockPos, Option<Rail>) {
        let position = self.vehicle.entity.pos.load();
        let current = BlockPos::floored_v(position);
        let below = match self.behavior {
            MinecartBehavior::Old => current.down(),
            MinecartBehavior::New(_) => BlockPos::new(
                current.0.x,
                (position.y - new_behavior::ON_RAIL_Y_OFFSET - EPSILON).floor() as i32,
                current.0.z,
            ),
        };
        let (block, state) = world.get_block_and_state_id(&below);
        if block.has_tag(&tag::Block::MINECRAFT_RAILS) {
            return (below, Rail::of(below, block, state));
        }
        if below == current {
            return (current, None);
        }
        let (block, state) = world.get_block_and_state_id(&current);
        (current, Rail::of(current, block, state))
    }

    /// Vanilla `Entity.applyGravity` with `AbstractMinecart.getDefaultGravity`.
    pub(super) fn apply_gravity(&self) {
        let entity = &self.vehicle.entity;
        if entity.has_no_gravity() {
            return;
        }
        let gravity = if entity.is_in_water() {
            IN_WATER_GRAVITY
        } else {
            GRAVITY
        };
        let mut velocity = entity.velocity.load();
        velocity.y -= gravity;
        entity.velocity.store(velocity);
    }

    /// Vanilla `AbstractMinecart.comeOffTrack`.
    pub(super) fn come_off_track(&self, caller: &dyn EntityBase) {
        let entity = &self.vehicle.entity;
        let max_speed = self.max_speed();
        let velocity = entity.velocity.load();
        let mut velocity = Vector3::new(
            velocity.x.clamp(-max_speed, max_speed),
            velocity.y,
            velocity.z.clamp(-max_speed, max_speed),
        );
        if entity.on_ground.load(Ordering::Relaxed) {
            velocity = velocity * 0.5;
        }
        entity.velocity.store(velocity);
        self.move_entity(caller, velocity);
        if !entity.on_ground.load(Ordering::Relaxed) {
            entity.velocity.store(entity.velocity.load() * AIR_DRAG);
        }
    }

    /// Vanilla `AbstractMinecart.getRedstoneDirection`: away from a solid block at the end of
    /// a powered straight rail.
    pub(super) fn redstone_direction(world: &World, rail: Rail) -> Vector3<f64> {
        if rail.powered_rail() != Some(true) {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        // Vanilla `isRedstoneConductor`.
        let conductor = |pos: BlockPos| world.get_block_state(&pos).is_solid_block();
        let pos = rail.pos;
        match rail.shape {
            RailShape::EastWest if conductor(pos.west()) => Vector3::new(1.0, 0.0, 0.0),
            RailShape::EastWest if conductor(pos.east()) => Vector3::new(-1.0, 0.0, 0.0),
            RailShape::NorthSouth if conductor(pos.north()) => Vector3::new(0.0, 0.0, 1.0),
            RailShape::NorthSouth if conductor(pos.south()) => Vector3::new(0.0, 0.0, -1.0),
            _ => Vector3::new(0.0, 0.0, 0.0),
        }
    }

    /// The driving player's move intent, `None` without input.
    pub(super) fn rider_intent(&self) -> Option<Vector3<f64>> {
        let passengers = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let intent = move_intent(passengers.first()?.get_player()?);
        (intent.length_squared() > 0.0).then_some(intent)
    }

    /// Vanilla `Entity.startRiding` (and `Mob.startRiding`) onto an empty cart, `true` when it
    /// boarded.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "callers pass the owned Arc from their entity query"
    )]
    pub(super) fn start_riding(
        entity: &Entity,
        world: &World,
        passenger: Arc<dyn EntityBase>,
    ) -> bool {
        let rider = passenger.get_entity();
        if rider.is_sneaking() || rider.riding_cooldown.load(Ordering::Relaxed) > 0 {
            return false;
        }
        let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
            return false;
        };
        entity.add_passenger(vehicle, passenger.clone());
        // Plugins may cancel the mount, so the rest only follows a boarding.
        if !entity.has_passenger(rider.entity_id) {
            return false;
        }
        rider.set_pose(EntityPose::Standing);
        world.emit_game_event_with_context(
            GameEvent::EntityMount.name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(passenger.as_ref()),
            },
        );
        if passenger.get_mob().is_some() {
            rider.drop_leash();
        }
        true
    }

    /// Farthest the cart can move along one axis in its next tick, whatever pushes add to its
    /// velocity first: rail steps run at the clamped track speed plus one slope gain and one
    /// boost, a move off the rails clamps each horizontal axis, and rail snaps add the rest.
    pub(crate) fn max_tick_travel(&self) -> f64 {
        let max_speed = self.behavior_max_speed();
        let track = (max_speed + new_behavior::POWERED_RAIL_START)
            * (1.0 + new_behavior::SLOPE_SPEED_SHARE)
            + new_behavior::SLIDE_SPEED;
        track + max_speed + RAIL_SNAP_REACH
    }

    /// How far the blast of a TNT cart can reach, `None` for other carts.
    pub(crate) fn explosion_reach(&self) -> Option<f64> {
        let MinecartKind::Tnt(minecart) = &self.kind else {
            return None;
        };
        // Explosions hurt entities within twice their power, past the blocks they break.
        Some(f64::from(minecart.max_explosion_power()) * 2.0)
    }

    /// Whether a TNT cart can explode during its own next tick: a burnt-down fuse, lava, or a
    /// hit or landing while it moves, falls or speeds up on a slope or powered rail.
    pub(crate) fn may_explode_next_tick(&self) -> bool {
        let MinecartKind::Tnt(minecart) = &self.kind else {
            return false;
        };
        let entity = &self.vehicle.entity;
        if minecart.fuse() == 0 || entity.touching_lava.load(Ordering::Relaxed) {
            return true;
        }
        let velocity = entity.velocity.load();
        let airborne =
            !entity.on_ground.load(Ordering::Relaxed) && !self.on_rails.load(Ordering::Relaxed);
        if velocity.horizontal_length() >= EPSILON || airborne {
            return true;
        }
        let (_, rail) = self.current_rail(&entity.world.load());
        rail.is_some_and(|rail| rail.shape.is_ascending() || rail.powered_rail() == Some(true))
    }
}
