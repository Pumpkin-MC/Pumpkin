use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::{
    Block, attributes::Attributes, effect::StatusEffect, entity::EntityPose, sound::Sound, tag,
};
use pumpkin_util::{
    math::{cos, lerp, position::BlockPos, sin, vector3::Vector3},
    random::{RandomImpl, xoroshiro128::Xoroshiro},
};
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};
use rand::{RngExt, seq::SliceRandom};

use super::{breeze_from_tick, look_from_eyes};
use crate::{
    entity::{
        Entity, EntityBase,
        ai::{
            brain::{
                BrainState, BrainTick,
                behavior::{
                    Behavior, BehaviorControl, MoveToTargetSink, Timed,
                    long_jump::calculate_jump_vector_for_angle,
                },
                memory::{MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget},
            },
            util::default_random_pos,
        },
        dismount_helper::is_block_dangerous,
        living::LivingEntity,
    },
    world::World,
};

const SPEED_MULTIPLIER_WHEN_SLIDING: f32 = 0.6;
const JUMP_CIRCLE_INNER_RADIUS: f32 = 4.0;
const JUMP_CIRCLE_MIDDLE_RADIUS: f32 = 8.0;
const JUMP_CIRCLE_DISTANCE_Y: f64 = 10.0;
const MAX_LINE_OF_SIGHT_TEST_RANGE: f64 = 50.0;
const REQUIRED_AIR_BLOCKS_ABOVE: i32 = 4;
const JUMP_COOLDOWN_TICKS: i64 = 10;
const JUMP_COOLDOWN_WHEN_HURT_TICKS: i64 = 2;
const INHALING_DURATION_TICKS: i64 = 10;
const MAX_JUMP_VELOCITY_MULTIPLIER: f32 = 0.058_333_334;
const ALLOWED_ANGLES: [i32; 5] = [40, 55, 60, 75, 80];

pub(super) fn long_jump() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(LongJump))
}

pub(super) fn slide() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(Slide))
}

pub(super) fn slide_to_target_sink() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(SlideToTargetSink(MoveToTargetSink::new(20, 40))))
}

fn follow_range(tick: &BrainTick<'_>) -> f64 {
    tick.mob
        .get_mob_entity()
        .living_entity
        .get_attribute_value(&Attributes::FOLLOW_RANGE)
}

pub(super) fn max_jump_speed(living: &LivingEntity) -> f64 {
    let follow_range = living.get_attribute_value(&Attributes::FOLLOW_RANGE) as f32;
    f64::from(MAX_JUMP_VELOCITY_MULTIPLIER * follow_range + living.get_jump_boost_power())
}

pub(super) fn block_read_radius(living: &LivingEntity) -> f64 {
    let follow_range = living.get_attribute_value(&Attributes::FOLLOW_RANGE);
    // Landing blocks can lie beyond the target, and retained targets can give sight rays longer
    // than the normal follow range. Include the landing block and the overhead sight check.
    MAX_LINE_OF_SIGHT_TEST_RANGE.max(follow_range + f64::from(JUMP_CIRCLE_MIDDLE_RADIUS) + 1.0)
}

fn random_point_behind_target(target: &Entity, random: &mut impl RngExt) -> Vector3<f64> {
    let mut random = Xoroshiro::from_seed(random.random());
    let view_angle = target.head_yaw.load() + 180.0 + random.next_gaussian() as f32 * 90.0 / 2.0;
    let distance = lerp(
        random.next_f32(),
        JUMP_CIRCLE_INNER_RADIUS,
        JUMP_CIRCLE_MIDDLE_RADIUS,
    );
    let angle = -view_angle * (std::f64::consts::PI / 180.0) as f32 - std::f32::consts::PI;
    let direction = Vector3::new(f64::from(-sin(angle)), 0.0, f64::from(-cos(angle)));
    target.pos.load() + direction * f64::from(distance)
}

fn normalize_direction(direction: Vector3<f64>) -> Vector3<f64> {
    const MIN_LENGTH: f32 = 1.0e-5;
    let length = direction.length();
    if length < f64::from(MIN_LENGTH) {
        Vector3::new(0.0, 0.0, 0.0)
    } else {
        direction / length
    }
}

fn has_line_of_sight(tick: &BrainTick<'_>, target: Vector3<f64>) -> bool {
    let from = tick.mob.get_entity().pos.load();
    let range = MAX_LINE_OF_SIGHT_TEST_RANGE.max(follow_range(tick));
    (target - from).length() <= range
        && tick
            .world
            .clip(&ClipContext::new(
                from,
                target,
                ClipBlock::Collider,
                ClipFluid::None,
            ))
            .is_miss()
}

fn snap_to_surface(world: &World, target: Vector3<f64>) -> Option<BlockPos> {
    for offset_y in [-10.0, 10.0] {
        let hit = world.clip(&ClipContext::new(
            target,
            target.add_raw(0.0, offset_y, 0.0),
            ClipBlock::Collider,
            ClipFluid::None,
        ));
        if hit.is_hit() {
            return Some(BlockPos::floored_v(hit.location).up());
        }
    }
    None
}

struct LongJump;

impl LongJump {
    fn clear_jump_memories(state: &mut BrainState) {
        state.erase(types::BREEZE_JUMP_TARGET.id());
        state.erase(types::BREEZE_JUMP_INHALING.id());
        state.erase(types::BREEZE_LEAVING_WATER.id());
    }

    fn can_jump_from_current_position(world: &World, position: BlockPos) -> bool {
        world.get_block(&position) != &Block::HONEY_BLOCK
            && (1..=REQUIRED_AIR_BLOCKS_ABOVE).all(|height| {
                let above = position.up_height(height);
                world.get_block_state(&above).is_air()
                    || world.fluid_state_has_tag(&above, &tag::Fluid::MINECRAFT_WATER)
            })
    }

    fn calculate_jump_velocity(tick: &BrainTick<'_>, target: BlockPos) -> Option<Vector3<f64>> {
        let living = &tick.mob.get_mob_entity().living_entity;
        let max_jump_velocity = MAX_JUMP_VELOCITY_MULTIPLIER * follow_range(tick) as f32;
        let mut angles = ALLOWED_ANGLES;
        angles.shuffle(&mut tick.mob.get_random());
        angles.into_iter().find_map(|angle| {
            let velocity = calculate_jump_vector_for_angle(
                tick.mob,
                target.to_f64(),
                max_jump_velocity,
                angle,
                false,
            )?;
            if living.has_effect(&StatusEffect::JUMP_BOOST) {
                let jump_boost =
                    normalize_direction(velocity).y * f64::from(living.get_jump_boost_power());
                Some(velocity.add_raw(0.0, jump_boost, 0.0))
            } else {
                Some(velocity)
            }
        })
    }
}

impl Behavior for LongJump {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 7] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::BREEZE_JUMP_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_INHALING.id(), MemoryStatus::Registered),
            (types::BREEZE_JUMP_TARGET.id(), MemoryStatus::Registered),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_LEAVING_WATER.id(), MemoryStatus::Registered),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        200
    }

    fn max_duration(&self) -> i32 {
        self.min_duration()
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let living = &tick.mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        if (!entity.on_ground.load(Relaxed) && !entity.is_in_water())
            || entity.water_height.load() > living.get_swim_height()
            || entity.touching_lava.load(Relaxed)
        {
            return false;
        }

        if tick.state.has_memory_value(types::BREEZE_JUMP_TARGET.id()) {
            return true;
        }
        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return false;
        };
        let target = target.get_entity();
        let range = follow_range(tick);
        let offset = target.pos.load() - entity.pos.load();
        if offset.length_squared() >= range * range {
            tick.state.erase(types::ATTACK_TARGET.id());
            return false;
        }
        let distance = Vector3::new(offset.x as f32, offset.y as f32, offset.z as f32).length();
        if distance <= JUMP_CIRCLE_INNER_RADIUS
            || !Self::can_jump_from_current_position(tick.world, entity.block_pos.load())
        {
            return false;
        }

        let candidate = random_point_behind_target(target, &mut tick.mob.get_random());
        let Some(landing) = snap_to_surface(tick.world, candidate) else {
            return false;
        };
        if is_block_dangerous(
            entity.entity_type,
            tick.world.get_block_state(&landing.down()),
        ) || !(has_line_of_sight(tick, landing.to_centered_f64())
            || has_line_of_sight(tick, landing.up_height(4).to_centered_f64()))
        {
            return false;
        }

        tick.state.set(types::BREEZE_JUMP_TARGET, landing);
        true
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        breeze_from_tick(tick).is_some_and(|breeze| breeze.planned_pose() != EntityPose::Standing)
            && !tick
                .state
                .has_memory_value(types::BREEZE_JUMP_COOLDOWN.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(breeze) = breeze_from_tick(tick) else {
            return;
        };
        if !tick
            .state
            .has_memory_value(types::BREEZE_JUMP_INHALING.id())
        {
            tick.state
                .set_with_expiry(types::BREEZE_JUMP_INHALING, (), INHALING_DURATION_TICKS);
        }
        let target = tick.state.get(types::BREEZE_JUMP_TARGET).copied();
        breeze.change_pose(EntityPose::Inhaling, move |breeze| {
            let entity = breeze.get_entity();
            entity
                .world
                .load()
                .play_sound_from_entity(breeze, Sound::EntityBreezeCharge);
            if let Some(target) = target {
                look_from_eyes(entity, target.to_centered_f64());
            }
        });
        breeze.after_brain_tick(|breeze| {
            if breeze.get_entity().pose.load() != EntityPose::Inhaling {
                breeze.mob_entity.with_brain(breeze, |brain| {
                    Self::clear_jump_memories(brain);
                });
            }
        });
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(breeze) = breeze_from_tick(tick) else {
            return;
        };
        let entity = breeze.get_entity();
        let in_water = entity.is_in_water();
        if !in_water {
            tick.state.erase(types::BREEZE_LEAVING_WATER.id());
        }

        let pose = breeze.planned_pose();
        if pose == EntityPose::Inhaling
            && !tick
                .state
                .has_memory_value(types::BREEZE_JUMP_INHALING.id())
        {
            let Some(velocity) = tick
                .state
                .get(types::BREEZE_JUMP_TARGET)
                .and_then(|&target| Self::calculate_jump_velocity(tick, target))
            else {
                breeze.change_pose(EntityPose::Standing, |_| {});
                return;
            };
            // Commit pose before playing transition sounds so a canceled pose stays silent.
            breeze.change_pose(EntityPose::LongJumping, move |breeze| {
                if in_water {
                    breeze.mob_entity.with_brain(breeze, |brain| {
                        brain.set(types::BREEZE_LEAVING_WATER, ());
                    });
                }
                breeze.play_entity_sound(Sound::EntityBreezeJump, 1.0, 1.0);
                let living = &breeze.mob_entity.living_entity;
                living.entity.yaw.store(living.entity.body_yaw.load());
                living.discard_friction.store(true, Relaxed);
                living.entity.velocity.store(velocity);
            });
        } else if pose == EntityPose::LongJumping
            && (entity.on_ground.load(Relaxed)
                || in_water
                    && !tick
                        .state
                        .has_memory_value(types::BREEZE_LEAVING_WATER.id()))
        {
            let cooldown = if tick.state.has_memory_value(types::HURT_BY.id()) {
                JUMP_COOLDOWN_WHEN_HURT_TICKS
            } else {
                JUMP_COOLDOWN_TICKS
            };
            breeze.change_pose(EntityPose::Standing, move |breeze| {
                breeze.play_entity_sound(Sound::EntityBreezeLand, 1.0, 1.0);
                breeze
                    .mob_entity
                    .living_entity
                    .discard_friction
                    .store(false, Relaxed);
                breeze.mob_entity.with_brain(breeze, |brain| {
                    brain.set_with_expiry(types::BREEZE_JUMP_COOLDOWN, (), cooldown);
                    brain.set_with_expiry(types::BREEZE_SHOOT, (), 100);
                });
            });
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(breeze) = breeze_from_tick(tick) {
            // An interrupted jump may never run the landing callback.
            breeze
                .mob_entity
                .living_entity
                .discard_friction
                .store(false, Relaxed);
            if matches!(
                breeze.planned_pose(),
                EntityPose::LongJumping | EntityPose::Inhaling
            ) {
                breeze.change_pose(EntityPose::Standing, |_| {});
            }
        }
        Self::clear_jump_memories(tick.state);
    }

    fn debug_name(&self) -> &'static str {
        "BreezeLongJump"
    }
}

struct Slide;

impl Behavior for Slide {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 4] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let entity = tick.mob.get_entity();
        entity.on_ground.load(Relaxed)
            && !entity.is_in_water()
            && breeze_from_tick(tick)
                .is_some_and(|breeze| breeze.planned_pose() == EntityPose::Standing)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return;
        };
        let target = target.get_entity();
        let target_pos = target.pos.load();
        let entity = tick.mob.get_entity();
        let pos = entity.pos.load();
        let center = entity.block_pos.load().to_centered_f64();
        let inner_radius = f64::from(JUMP_CIRCLE_INNER_RADIUS);
        let route = if center.squared_distance_to_vec_xz(target_pos) < inner_radius * inner_radius
            && (target_pos.y - center.y).abs() < JUMP_CIRCLE_DISTANCE_Y
        {
            default_random_pos::get_pos_away(tick.mob, 5, 5, target_pos).filter(|&away| {
                has_line_of_sight(tick, away)
                    && target_pos.squared_distance_to_vec(&away)
                        > target_pos.squared_distance_to_vec(&pos)
            })
        } else {
            None
        };
        let destination = route.unwrap_or_else(|| {
            let mut random = tick.mob.get_random();
            if random.random() {
                return random_point_behind_target(target, &mut random);
            }
            let direction = target_pos - pos;
            let distance = direction.length()
                - lerp(
                    random.random::<f64>(),
                    f64::from(JUMP_CIRCLE_MIDDLE_RADIUS),
                    inner_radius,
                );
            pos + normalize_direction(direction) * distance
        });
        tick.state.set(
            types::WALK_TARGET,
            WalkTarget::from_vec(destination, SPEED_MULTIPLIER_WHEN_SLIDING, 1),
        );
    }

    fn debug_name(&self) -> &'static str {
        "BreezeSlide"
    }
}

struct SlideToTargetSink(MoveToTargetSink);

impl SlideToTargetSink {
    fn begin_sliding(tick: &BrainTick<'_>) {
        if let Some(breeze) = breeze_from_tick(tick) {
            breeze.change_pose(EntityPose::Sliding, |breeze| {
                breeze.play_entity_sound(Sound::EntityBreezeSlide, 1.0, 1.0);
            });
        }
    }
}

impl Behavior for SlideToTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        self.0.entry_conditions()
    }

    fn min_duration(&self) -> i32 {
        self.0.min_duration()
    }

    fn max_duration(&self) -> i32 {
        self.0.max_duration()
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.check_extra_start_conditions(tick)
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.can_still_use(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.0.start(tick);
        Self::begin_sliding(tick);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if self.0.tick_path(tick) {
            Self::begin_sliding(tick);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        self.0.stop(tick);
        if let Some(breeze) = breeze_from_tick(tick) {
            let has_target = tick.state.has_memory_value(types::ATTACK_TARGET.id());
            breeze.change_pose(EntityPose::Standing, move |breeze| {
                if has_target {
                    breeze.mob_entity.with_brain(breeze, |brain| {
                        brain.set_with_expiry(types::BREEZE_SHOOT, (), 60);
                    });
                }
            });
        }
    }

    fn debug_name(&self) -> &'static str {
        "BreezeSlideToTargetSink"
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::entity::EntityType;

    use super::super::BreezeEntity;
    use super::*;
    use crate::entity::ai::{brain::behavior::Status, test_support};

    #[tokio::test]
    async fn expired_long_jump_restores_friction_before_committing_the_pose()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BREEZE,
        ));
        let living = &breeze.mob_entity.living_entity;
        living.entity.pose.store(EntityPose::LongJumping);
        living.discard_friction.store(true, Relaxed);

        let mut state = BrainState::default();
        let mut jump = Timed::new(LongJump);
        let duration = jump.inner().max_duration();
        jump.start_at(0, duration);
        jump.tick_or_stop(&mut BrainTick {
            mob: breeze.as_ref(),
            world: &world,
            state: &mut state,
            time: i64::from(duration) + 1,
        });
        world.level.shutdown().await;

        assert_eq!(jump.status(), Status::Stopped);
        assert_eq!(living.entity.pose.load(), EntityPose::LongJumping);
        assert!(!living.discard_friction.load(Relaxed));
        Ok(())
    }
}
