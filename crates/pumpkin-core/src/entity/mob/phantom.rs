use std::f32::consts::PI;
use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{approach, approach_degrees, degrees_difference_abs, wrap_degrees};
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::entity::ai::brain::memory::codec::{block_pos_from_tag, block_pos_to_tag};
use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::passive::cat::CatEntity;
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase,
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::World;

/// Vanilla `Phantom.TICKS_PER_FLAP`, `Mth.ceil(24.166098F)`.
const TICKS_PER_FLAP: i32 = 25;
const MAX_SIZE: i32 = 64;
const SIZE_SCALE: f32 = 0.15;
const BASE_ATTACK_DAMAGE: i32 = 6;
const ANCHOR_SPAWN_HEIGHT: i32 = 5;
const TARGET_SEARCH_RANGE: f64 = 64.0;
const TARGET_SEARCH_XZ: f64 = 16.0;
const CAT_SEARCH_RANGE: f64 = 16.0;
const CAT_SEARCH_TICK_DELAY: i32 = 20;
const SWOOP_SOUND_VOLUME: f32 = 10.0;
const TOUCHING_TARGET_DISTANCE_SQ: f64 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum AttackPhase {
    #[default]
    Circle,
    Swoop,
}

fn as_phantom(mob: &dyn Mob) -> Option<&PhantomEntity> {
    mob.cast_any().downcast_ref::<PhantomEntity>()
}

pub struct PhantomEntity {
    pub mob_entity: MobEntity,
    move_target_point: AtomicCell<Vector3<f64>>,
    anchor_point: AtomicCell<Option<BlockPos>>,
    attack_phase: AtomicCell<AttackPhase>,
    size: AtomicI32,
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(PhantomMoveControl::default());
        let phantom = Self {
            mob_entity,
            move_target_point: AtomicCell::new(Vector3::default()),
            anchor_point: AtomicCell::new(None),
            attack_phase: AtomicCell::new(AttackPhase::Circle),
            size: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(phantom);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            goal_selector.add_goal(1, Box::new(PhantomAttackStrategyGoal::default()));
            goal_selector.add_goal(2, Box::new(PhantomSweepAttackGoal::default()));
            goal_selector.add_goal(3, Box::new(PhantomCircleAroundAnchorGoal::default()));
            mob_arc
                .mob_entity
                .target_selector
                .add_goal(1, Box::new(PhantomAttackPlayerTargetGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_phantom_size(&self) -> i32 {
        self.size.load(Ordering::Relaxed)
    }

    /// Only a real change resizes the phantom and resets its attack damage.
    pub fn set_phantom_size(&self, size: i32) {
        let size = size.clamp(0, MAX_SIZE);
        if self.size.swap(size, Ordering::Relaxed) == size {
            return;
        }
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::phantom::ID_SIZE, size);
        let scale = SIZE_SCALE.mul_add(size as f32, 1.0);
        let base = Entity::type_dimensions(entity.entity_type);
        entity.set_default_dimensions(EntityDimensions {
            width: base.width * scale,
            height: base.height * scale,
            eye_height: base.eye_height * scale,
        });
        self.mob_entity.living_entity.set_attribute_base(
            &Attributes::ATTACK_DAMAGE,
            f64::from(BASE_ATTACK_DAMAGE + size),
        );
    }

    fn attack_phase(&self) -> AttackPhase {
        self.attack_phase.load()
    }

    fn set_attack_phase(&self, phase: AttackPhase) {
        self.attack_phase.store(phase);
    }

    /// `TargetingConditions.DEFAULT`.
    fn can_attack_target(&self, target: &dyn EntityBase) -> bool {
        let world = self.get_entity().world.load();
        TargetPredicate::create_attackable().test(&world, Some(self), target)
    }
}

impl Mob for PhantomEntity {
    fn mob_is_flapping(&self) -> bool {
        let entity = self.get_entity();
        // Vanilla `getUniqueFlapTickOffset` spreads the flaps by entity id.
        entity
            .entity_id
            .wrapping_mul(3)
            .wrapping_add(entity.tick_count.load(Ordering::Relaxed))
            % TICKS_PER_FLAP
            == 0
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn on_climbable(&self) -> bool {
        false
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn should_check_fall_damage(&self) -> bool {
        false
    }

    fn travel(&self, caller: &dyn EntityBase) -> bool {
        self.mob_entity
            .living_entity
            .travel_flying_with_speed(caller, 0.2);
        true
    }

    fn ticks_look_control(&self) -> bool {
        false
    }

    fn ticks_body_rotation(&self) -> bool {
        false
    }

    fn mob_ai_step(&self) {
        let entity = self.get_entity();
        entity.head_yaw.store(entity.body_yaw.load());
        entity.body_yaw.store(entity.yaw.load());
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.anchor_point.store(Some(
            self.get_entity()
                .block_pos
                .load()
                .up_height(ANCHOR_SPAWN_HEIGHT),
        ));
        self.set_phantom_size(0);
        self.default_finalize_spawn(world, group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(anchor) = self.anchor_point.load() {
            nbt.put("anchor_pos", block_pos_to_tag(anchor));
        }
        nbt.put_int("size", self.get_phantom_size());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.anchor_point
            .store(nbt.get("anchor_pos").and_then(block_pos_from_tag));
        self.set_phantom_size(nbt.get_int("size").unwrap_or(0));
    }
}

/// Turns at most 4 degrees a tick, speeding up on straight lines and slowing in turns.
struct PhantomMoveControl {
    inner: MoveControl,
    speed: f32,
}

impl Default for PhantomMoveControl {
    fn default() -> Self {
        Self {
            inner: MoveControl::default(),
            speed: 0.1,
        }
    }
}

impl PhantomMoveControl {
    const MAX_TURN: f32 = 4.0;
    const STRAIGHT_TURN: f32 = 3.0;
    const MAX_SPEED: f32 = 1.8;
    const TURN_SPEED: f32 = 0.2;
}

impl Control for PhantomMoveControl {}

impl MoveControlTrait for PhantomMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        let entity = phantom.get_entity();
        if entity.horizontal_collision.load(Ordering::Relaxed) {
            entity.yaw.store(entity.yaw.load() + 180.0);
            self.speed = 0.1;
        }

        let pos = entity.pos.load();
        let target = phantom.move_target_point.load();
        let mut tdx = target.x - pos.x;
        let tdy = target.y - pos.y;
        let mut tdz = target.z - pos.z;
        let mut sd = tdx.hypot(tdz);
        if sd.abs() <= f64::from(1.0E-5f32) {
            return;
        }
        let y_relative_scale = 1.0 - (tdy * f64::from(0.7f32)).abs() / sd;
        tdx *= y_relative_scale;
        tdz *= y_relative_scale;
        sd = tdx.hypot(tdz);
        let sd2 = (tdx * tdx + tdz * tdz + tdy * tdy).sqrt();
        let prev = entity.yaw.load();
        let angle = tdz.atan2(tdx) as f32;
        let a = wrap_degrees(prev + 90.0);
        let b = wrap_degrees(angle.to_degrees());
        let yaw = approach_degrees(a, b, Self::MAX_TURN) - 90.0;
        entity.yaw.store(yaw);
        entity.body_yaw.store(yaw);
        self.speed = if degrees_difference_abs(prev, yaw) < Self::STRAIGHT_TURN {
            approach(
                self.speed,
                Self::MAX_SPEED,
                0.005 * (Self::MAX_SPEED / self.speed),
            )
        } else {
            approach(self.speed, Self::TURN_SPEED, 0.025)
        };

        let x_rot = -((-tdy).atan2(sd) as f32).to_degrees();
        entity.pitch.store(x_rot);
        let move_angle = (yaw + 90.0).to_radians();
        let speed = f64::from(self.speed);
        let txd = speed * f64::from(move_angle.cos()) * (tdx / sd2).abs();
        let tzd = speed * f64::from(move_angle.sin()) * (tdz / sd2).abs();
        let tyd = speed * f64::from(x_rot.to_radians().sin()) * (tdy / sd2).abs();
        let movement = entity.velocity.load();
        entity
            .velocity
            .store(movement + (Vector3::new(txd, tyd, tzd) - movement) * 0.2);
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.inner.set_wanted_position(x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forwards: f32, right: f32) {
        self.inner.strafe(forwards, right);
    }

    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.inner.get_wanted_position()
    }
}

/// Picks the highest player around as the target.
struct PhantomAttackPlayerTargetGoal {
    next_scan_tick: i32,
}

impl Default for PhantomAttackPlayerTargetGoal {
    fn default() -> Self {
        Self {
            next_scan_tick: to_goal_ticks(20),
        }
    }
}

impl Goal for PhantomAttackPlayerTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.next_scan_tick > 0 {
            self.next_scan_tick -= 1;
            return false;
        }
        self.next_scan_tick = to_goal_ticks(60);
        let Some(phantom) = as_phantom(mob) else {
            return false;
        };
        let entity = phantom.get_entity();
        let world = entity.world.load();
        let search = entity.bounding_box.load().expand(
            TARGET_SEARCH_XZ,
            TARGET_SEARCH_RANGE,
            TARGET_SEARCH_XZ,
        );
        let in_range =
            TargetPredicate::create_attackable().set_base_max_distance(TARGET_SEARCH_RANGE);
        let mut players: Vec<_> = world
            .get_players_at_box(&search)
            .into_iter()
            .filter(|player| in_range.test(&world, Some(phantom), player.as_ref()))
            .collect();
        players.sort_by(|a, b| {
            b.get_entity()
                .pos
                .load()
                .y
                .total_cmp(&a.get_entity().pos.load().y)
        });
        let Some(player) = players
            .into_iter()
            .find(|player| phantom.can_attack_target(player.as_ref()))
        else {
            return false;
        };
        let target: Arc<dyn EntityBase> = player;
        phantom.set_mob_target(Some(target), TargetReason::ClosestPlayer);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        as_phantom(mob).is_some_and(|phantom| {
            phantom
                .mob_entity
                .get_target()
                .is_some_and(|target| phantom.can_attack_target(target.as_ref()))
        })
    }
}

/// Circles above the target and switches to a swoop every 8 to 12 seconds.
#[derive(Default)]
struct PhantomAttackStrategyGoal {
    next_sweep_tick: i32,
}

impl PhantomAttackStrategyGoal {
    fn set_anchor_above_target(phantom: &PhantomEntity) {
        if phantom.anchor_point.load().is_none() {
            return;
        }
        let Some(target) = phantom.mob_entity.get_target() else {
            return;
        };
        let mut anchor = target
            .get_entity()
            .block_pos
            .load()
            .up_height(20 + phantom.get_random().random_range(0..20));
        let sea_level = phantom.get_entity().world.load().sea_level;
        if anchor.0.y < sea_level {
            anchor.0.y = sea_level + 1;
        }
        phantom.anchor_point.store(Some(anchor));
    }
}

impl Goal for PhantomAttackStrategyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_phantom(mob).is_some_and(|phantom| {
            phantom
                .mob_entity
                .get_target()
                .is_some_and(|target| phantom.can_attack_target(target.as_ref()))
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.next_sweep_tick = self.get_tick_count(10);
        if let Some(phantom) = as_phantom(mob) {
            phantom.set_attack_phase(AttackPhase::Circle);
            Self::set_anchor_above_target(phantom);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        let Some(anchor) = phantom.anchor_point.load() else {
            return;
        };
        let world = phantom.get_entity().world.load();
        let height =
            world.get_heightmap_height(ChunkHeightmapType::MotionBlocking, anchor.0.x, anchor.0.z)
                + 1;
        let rise = 10 + phantom.get_random().random_range(0..20);
        phantom
            .anchor_point
            .store(Some(BlockPos::new(anchor.0.x, height + rise, anchor.0.z)));
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        if phantom.attack_phase() != AttackPhase::Circle {
            return;
        }
        self.next_sweep_tick -= 1;
        if self.next_sweep_tick > 0 {
            return;
        }
        phantom.set_attack_phase(AttackPhase::Swoop);
        Self::set_anchor_above_target(phantom);
        let mut rng = phantom.get_random();
        let delay = (8 + rng.random_range(0..4)) * 20;
        let pitch = rng.random::<f32>().mul_add(0.1, 0.95);
        drop(rng);
        self.next_sweep_tick = self.get_tick_count(delay);
        phantom.play_entity_sound(Sound::EntityPhantomSwoop, SWOOP_SOUND_VOLUME, pitch);
    }
}

/// Flies in circles of 5 to 15 blocks around the anchor point.
#[derive(Default)]
struct PhantomCircleAroundAnchorGoal {
    angle: f32,
    distance: f32,
    height: f32,
    clockwise: f32,
}

impl PhantomCircleAroundAnchorGoal {
    fn select_next(&mut self, phantom: &PhantomEntity) {
        let anchor = phantom.anchor_point.load().unwrap_or_else(|| {
            let pos = phantom.get_entity().block_pos.load();
            phantom.anchor_point.store(Some(pos));
            pos
        });
        self.angle += self.clockwise * 15.0f32.to_radians();
        phantom.move_target_point.store(Vector3::new(
            f64::from(anchor.0.x) + f64::from(self.distance * self.angle.cos()),
            f64::from(anchor.0.y) + f64::from(-4.0 + self.height),
            f64::from(anchor.0.z) + f64::from(self.distance * self.angle.sin()),
        ));
    }
}

impl Goal for PhantomCircleAroundAnchorGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_phantom(mob).is_some_and(|phantom| {
            phantom.mob_entity.get_target().is_none()
                || phantom.attack_phase() == AttackPhase::Circle
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        let mut rng = phantom.get_random();
        self.distance = rng.random::<f32>().mul_add(10.0, 5.0);
        self.height = rng.random::<f32>().mul_add(9.0, -4.0);
        self.clockwise = if rng.random::<bool>() { 1.0 } else { -1.0 };
        drop(rng);
        self.select_next(phantom);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        {
            let mut rng = phantom.get_random();
            if rng.random_range(0..self.get_tick_count(350)) == 0 {
                self.height = rng.random::<f32>().mul_add(9.0, -4.0);
            }
            if rng.random_range(0..self.get_tick_count(250)) == 0 {
                self.distance += 1.0;
                if self.distance > 15.0 {
                    self.distance = 5.0;
                    self.clockwise = -self.clockwise;
                }
            }
            if rng.random_range(0..self.get_tick_count(450)) == 0 {
                self.angle = rng.random::<f32>() * 2.0 * PI;
                drop(rng);
                self.select_next(phantom);
            }
        }

        let entity = phantom.get_entity();
        if phantom
            .move_target_point
            .load()
            .squared_distance_to_vec(&entity.pos.load())
            < TOUCHING_TARGET_DISTANCE_SQ
        {
            self.select_next(phantom);
        }

        let world = entity.world.load();
        let block_pos = entity.block_pos.load();
        let y = entity.pos.load().y;
        if phantom.move_target_point.load().y < y
            && !world.get_block_state(&block_pos.down()).is_air()
        {
            self.height = self.height.max(1.0);
            self.select_next(phantom);
        }
        if phantom.move_target_point.load().y > y
            && !world.get_block_state(&block_pos.up()).is_air()
        {
            self.height = self.height.min(-1.0);
            self.select_next(phantom);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

/// Dives at the target; nearby cats hiss and scare it off.
#[derive(Default)]
struct PhantomSweepAttackGoal {
    is_scared_of_cat: bool,
    cat_search_tick: i32,
}

impl Goal for PhantomSweepAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_phantom(mob).is_some_and(|phantom| {
            phantom.mob_entity.get_target().is_some()
                && phantom.attack_phase() == AttackPhase::Swoop
        })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(phantom) = as_phantom(mob) else {
            return false;
        };
        let Some(target) = phantom.mob_entity.get_target() else {
            return false;
        };
        if !target.get_entity().is_alive()
            || target.is_spectator()
            || target.get_player().is_some_and(Player::is_creative)
            || !self.can_start(mob)
        {
            return false;
        }
        let entity = phantom.get_entity();
        let tick_count = entity.tick_count.load(Ordering::Relaxed);
        if tick_count > self.cat_search_tick {
            self.cat_search_tick = tick_count + CAT_SEARCH_TICK_DELAY;
            let world = entity.world.load();
            let area = entity.bounding_box.load().expand_all(CAT_SEARCH_RANGE);
            let cats: Vec<_> = world
                .get_entities_of_type_at_box(&area, |entity_type| entity_type == &EntityType::CAT)
                .into_iter()
                .filter(|cat| cat.get_entity().is_alive())
                .collect();
            for cat in &cats {
                if let Some(cat) = cat.cast_any().downcast_ref::<CatEntity>() {
                    cat.hiss();
                }
            }
            self.is_scared_of_cat = !cats.is_empty();
        }
        !self.is_scared_of_cat
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(phantom) = as_phantom(mob) {
            phantom.set_mob_target(None, TargetReason::Unknown);
            phantom.set_attack_phase(AttackPhase::Circle);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = as_phantom(mob) else {
            return;
        };
        let Some(target) = phantom.mob_entity.get_target() else {
            return;
        };
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        phantom.move_target_point.store(Vector3::new(
            target_pos.x,
            target_pos.y + f64::from(target_entity.height()) * 0.5,
            target_pos.z,
        ));
        let entity = phantom.get_entity();
        if entity
            .bounding_box
            .load()
            .expand_all(f64::from(0.2f32))
            .intersects(&target_entity.bounding_box.load())
        {
            phantom.do_hurt_target(target.as_ref());
            phantom.set_attack_phase(AttackPhase::Circle);
            if !entity.is_silent() {
                entity.world.load().sync_world_event(
                    WorldEvent::SoundPhantomBite,
                    entity.block_pos.load(),
                    0,
                );
            }
        } else if entity.horizontal_collision.load(Ordering::Relaxed)
            || phantom.mob_entity.living_entity.was_hurt_recently()
        {
            phantom.set_attack_phase(AttackPhase::Circle);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
