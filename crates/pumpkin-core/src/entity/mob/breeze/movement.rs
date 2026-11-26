use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::{
    Block, attributes::Attributes, effect::StatusEffect, entity::EntityPose, sound::Sound, tag,
};
use pumpkin_util::{
    math::{cos, lerp, position::BlockPos, sin, vector3::Vector3},
    random::RandomImpl,
};
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};

use super::{BreezeEntity, look_from_eyes};
use crate::{
    entity::{
        Entity,
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
    Box::new(Timed::new(LongJump::default()))
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

fn random_point_behind_target(target: &Entity, random: &mut impl RandomImpl) -> Vector3<f64> {
    let spread = random.next_gaussian();
    let view_angle = target.head_yaw.load() + 180.0 + spread as f32 * 90.0 / 2.0;
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
            // Java floors the hit point before moving up, even at an integer surface.
            return Some(BlockPos::floored_v(hit.location).up());
        }
    }
    None
}

#[derive(Default)]
struct LongJump {
    took_off: bool,
}

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
        let breeze = tick.mob.cast_any().downcast_ref::<BreezeEntity>()?;
        let living = &tick.mob.get_mob_entity().living_entity;
        let max_jump_velocity = MAX_JUMP_VELOCITY_MULTIPLIER * follow_range(tick) as f32;
        let mut angles = ALLOWED_ANGLES;
        breeze.with_movement_random(|random| {
            for count in (2..=angles.len()).rev() {
                let other = random.next_bounded_i32(count as i32) as usize;
                angles.swap(count - 1, other);
            }
        });
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
            super::ai::forget_target(tick);
            return false;
        }
        let distance = Vector3::new(offset.x as f32, offset.y as f32, offset.z as f32).length();
        if distance <= JUMP_CIRCLE_INNER_RADIUS
            || !Self::can_jump_from_current_position(tick.world, entity.block_pos.load())
        {
            return false;
        }

        let Some(breeze) = tick.mob.cast_any().downcast_ref::<BreezeEntity>() else {
            return false;
        };
        let candidate =
            breeze.with_movement_random(|random| random_point_behind_target(target, random));
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
        tick.pose() != EntityPose::Standing
            && !tick
                .state
                .has_memory_value(types::BREEZE_JUMP_COOLDOWN.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if !tick
            .state
            .has_memory_value(types::BREEZE_JUMP_INHALING.id())
        {
            tick.state
                .set_with_expiry(types::BREEZE_JUMP_INHALING, (), INHALING_DURATION_TICKS);
        }
        tick.set_pose(EntityPose::Inhaling);
        let target = tick.state.get(types::BREEZE_JUMP_TARGET).copied();
        tick.defer(move |breeze| {
            let entity = breeze.get_entity();
            entity.world.load().play_sound(
                Sound::EntityBreezeCharge,
                breeze.sound_source(),
                &entity.pos.load(),
            );
            if let Some(target) = target {
                look_from_eyes(entity, target.to_centered_f64());
            }
        });
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let entity = tick.mob.get_entity();
        let in_water = entity.is_in_water();
        if !in_water {
            tick.state.erase(types::BREEZE_LEAVING_WATER.id());
        }

        let pose = tick.pose();
        if !self.took_off
            && pose == EntityPose::Inhaling
            && !tick
                .state
                .has_memory_value(types::BREEZE_JUMP_INHALING.id())
        {
            let Some(velocity) = tick
                .state
                .get(types::BREEZE_JUMP_TARGET)
                .and_then(|&target| Self::calculate_jump_velocity(tick, target))
            else {
                tick.set_pose(EntityPose::Standing);
                return;
            };
            if in_water {
                tick.state.set(types::BREEZE_LEAVING_WATER, ());
            }
            self.took_off = true;
            tick.defer(|breeze| breeze.play_entity_sound(Sound::EntityBreezeJump, 1.0, 1.0));
            tick.set_pose(EntityPose::LongJumping);
            tick.defer(move |mob| {
                let living = &mob.get_mob_entity().living_entity;
                let entity = &living.entity;
                entity.yaw.store(entity.body_yaw.load());
                living.discard_friction.store(true, Relaxed);
                entity.velocity.store(velocity);
            });
        } else if (self.took_off || pose == EntityPose::LongJumping)
            && (entity.on_ground.load(Relaxed)
                || in_water
                    && !tick
                        .state
                        .has_memory_value(types::BREEZE_LEAVING_WATER.id()))
        {
            tick.defer(|breeze| breeze.play_entity_sound(Sound::EntityBreezeLand, 1.0, 1.0));
            tick.set_pose(EntityPose::Standing);
            tick.mob
                .get_mob_entity()
                .living_entity
                .discard_friction
                .store(false, Relaxed);
            let cooldown = if tick.state.has_memory_value(types::HURT_BY.id()) {
                JUMP_COOLDOWN_WHEN_HURT_TICKS
            } else {
                JUMP_COOLDOWN_TICKS
            };
            tick.state
                .set_with_expiry(types::BREEZE_JUMP_COOLDOWN, (), cooldown);
            tick.state.set_with_expiry(types::BREEZE_SHOOT, (), 100);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        self.took_off = false;
        if matches!(tick.pose(), EntityPose::LongJumping | EntityPose::Inhaling) {
            tick.set_pose(EntityPose::Standing);
        }
        // An interrupted jump may never land.
        tick.mob
            .get_mob_entity()
            .living_entity
            .discard_friction
            .store(false, Relaxed);
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
            && tick.pose() == EntityPose::Standing
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(breeze) = tick.mob.cast_any().downcast_ref::<BreezeEntity>() else {
            return;
        };
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
            breeze.with_movement_random(|random| {
                if random.next_bool() {
                    return random_point_behind_target(target, random);
                }
                let direction = target_pos - pos;
                let distance = direction.length()
                    - lerp(
                        random.next_f64(),
                        f64::from(JUMP_CIRCLE_MIDDLE_RADIUS),
                        inner_radius,
                    );
                pos + normalize_direction(direction) * distance
            })
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
    fn begin_sliding(tick: &mut BrainTick<'_>) {
        tick.defer(|breeze| breeze.play_entity_sound(Sound::EntityBreezeSlide, 1.0, 1.0));
        tick.set_pose(EntityPose::Sliding);
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

    fn stop(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        self.0.stop(tick, timed_out);
        tick.set_pose(EntityPose::Standing);
        if tick.state.has_memory_value(types::ATTACK_TARGET.id()) {
            tick.state.set_with_expiry(types::BREEZE_SHOOT, (), 60);
        }
    }

    fn debug_name(&self) -> &'static str {
        "BreezeSlideToTargetSink"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicUsize};

    use futures::future::BoxFuture;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;

    use super::*;
    use crate::entity::ai::{
        brain::{Brain, activity::ActivityData, behavior::Status},
        test_support,
    };
    use crate::entity::{EntityBase, mob::Mob, r#type::from_type};
    use crate::plugin::api::events::entity::entity_pose_change::EntityPoseChangeEvent;
    use crate::plugin::{EventHandler, EventPriority};
    use crate::server::Server;

    struct VetoPose {
        entity_id: i32,
        pose: EntityPose,
        attempts: AtomicUsize,
    }

    impl EventHandler<EntityPoseChangeEvent> for VetoPose {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityPoseChangeEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.entity_id == self.entity_id && event.pose.parse() == Ok(self.pose as u8) {
                    self.attempts.fetch_add(1, Relaxed);
                    event.cancelled = true;
                }
            })
        }
    }

    #[tokio::test]
    async fn a_breeze_reuses_its_second_gaussian_sample() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let position = Vector3::new(0.0, 64.0, 0.0);
        let breeze = BreezeEntity::new(Entity::new(world.clone(), position, &EntityType::BREEZE));
        let other = BreezeEntity::new(Entity::new(world.clone(), position, &EntityType::BREEZE));
        let target = Entity::new(world.clone(), position, &EntityType::IRON_GOLEM);
        target.head_yaw.store(0.0);
        breeze.with_movement_random(|random| {
            *random = pumpkin_util::random::legacy_rand::LegacyRand::from_seed(26);
        });
        other.with_movement_random(|random| {
            *random = pumpkin_util::random::legacy_rand::LegacyRand::from_seed(93);
        });

        // Java 26.3's BreezeUtil calculation with the two legacy random sources interleaved.
        for (breeze, x, z) in [
            (&breeze, 7.524_081_344_914_464, -1.805_914_356_706_594_5),
            (&other, 5.433_498_017_023_055, -1.729_926_241_146_770_8),
            (&breeze, 1.065_079_664_289_896_8, -4.009_766_138_940_904),
        ] {
            let point =
                breeze.with_movement_random(|random| random_point_behind_target(&target, random));
            assert!(
                point.squared_distance_to_vec(&Vector3::new(x, 64.0, z)) < 1e-10,
                "{point:?}"
            );
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn slide_stop_arms_a_shot_even_when_standing_is_vetoed()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let position = Vector3::new(0.0, 64.0, 0.0);
        let breeze = BreezeEntity::new(Entity::new(world.clone(), position, &EntityType::BREEZE));
        let target = from_type(
            &EntityType::IRON_GOLEM,
            position,
            &world,
            uuid::Uuid::new_v4(),
        );
        let body = breeze.get_mob_entity();
        let mut brain = Brain::default();
        brain.register_memory(types::ATTACK_TARGET.id());
        brain.register_memory(types::BREEZE_SHOOT.id());
        brain.set(types::ATTACK_TARGET, target);
        body.set_brain(brain);
        let observer = Arc::new(VetoPose {
            entity_id: breeze.get_entity().entity_id,
            pose: EntityPose::Standing,
            attempts: AtomicUsize::new(0),
        });
        server.plugin_manager.register::<EntityPoseChangeEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        breeze.get_entity().set_pose(EntityPose::Sliding);
        let effects = body.with_brain(breeze.as_ref(), |brain| {
            assert!(!brain.has_memory_value(types::BREEZE_SHOOT.id()));
            SlideToTargetSink(MoveToTargetSink::new(20, 40)).stop(
                &mut BrainTick {
                    mob: breeze.as_ref(),
                    world: &world,
                    state: brain,
                    time: 0,
                },
                false,
            );
            brain.take_effects()
        });
        for effect in effects {
            effect(breeze.as_ref());
        }
        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Sliding);
        body.with_brain(breeze.as_ref(), |brain| {
            assert_eq!(brain.time_until_expiry(types::BREEZE_SHOOT.id()), Some(60));
        });
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn expired_long_jump_restores_friction_even_when_standing_is_vetoed()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BREEZE,
        ));
        let observer = Arc::new(VetoPose {
            entity_id: breeze.get_entity().entity_id,
            pose: EntityPose::Standing,
            attempts: AtomicUsize::new(0),
        });
        server.plugin_manager.register::<EntityPoseChangeEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let body = breeze.get_mob_entity();
        let living = &breeze.mob_entity.living_entity;
        living.entity.set_pose(EntityPose::LongJumping);
        living.discard_friction.store(true, Relaxed);
        assert!(!living.entity.on_ground.load(Relaxed));

        let mut jump = Timed::new(LongJump::default());
        let duration = jump.inner().max_duration();
        jump.start_at(0, duration);
        let effects = body.with_brain(breeze.as_ref(), |brain| {
            jump.tick_or_stop(&mut BrainTick {
                mob: breeze.as_ref(),
                world: &world,
                state: brain,
                time: i64::from(duration) + 1,
            });
            brain.take_effects()
        });
        assert_eq!(jump.status(), Status::Stopped);
        assert!(!living.discard_friction.load(Relaxed));
        for effect in effects {
            effect(breeze.as_ref());
        }
        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(living.entity.pose.load(), EntityPose::LongJumping);
        breeze.custom_server_ai_step(breeze.as_ref());
        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(living.entity.pose.load(), EntityPose::LongJumping);
        assert!(!body.brain_has_memory_value(types::BREEZE_SHOOT.id()));
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn slide_sees_the_pose_after_the_jump_charge_is_vetoed()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 300.0, 0.5),
            &EntityType::BREEZE,
        ));
        let body = &breeze.mob_entity;
        let entity = breeze.get_entity();
        entity.on_ground.store(true, Relaxed);
        let target = from_type(
            &EntityType::IRON_GOLEM,
            Vector3::new(8.5, 300.0, 0.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        let observer = Arc::new(VetoPose {
            entity_id: entity.entity_id,
            pose: EntityPose::Inhaling,
            attempts: AtomicUsize::new(0),
        });
        server.plugin_manager.register::<EntityPoseChangeEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![long_jump(), slide()],
        ));
        brain.set(types::ATTACK_TARGET, target);
        brain.set(types::BREEZE_JUMP_TARGET, BlockPos::new(8, 300, 0));
        body.set_brain(brain);
        assert!(!body.brain_has_memory_value(types::WALK_TARGET.id()));

        body.tick_brain(breeze.as_ref());

        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(entity.pose.load(), EntityPose::Standing);
        assert!(body.brain_has_memory_value(types::WALK_TARGET.id()));
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_breeze_takes_off_even_when_its_jumping_pose_is_vetoed()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 300.0, 0.5),
            &EntityType::BREEZE,
        ));
        let body = &breeze.mob_entity;
        let entity = &body.living_entity.entity;
        entity.set_pose(EntityPose::Inhaling);
        assert_eq!(entity.velocity.load(), Vector3::default());
        assert!(!body.living_entity.discard_friction.load(Relaxed));
        let observer = Arc::new(VetoPose {
            entity_id: entity.entity_id,
            pose: EntityPose::LongJumping,
            attempts: AtomicUsize::new(0),
        });
        server.plugin_manager.register::<EntityPoseChangeEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let mut brain = Brain::default();
        let mut jump = Timed::new(LongJump::default());
        jump.start_at(world.get_world_age(), jump.inner().max_duration());
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![Box::new(jump)],
        ));
        brain.set(types::BREEZE_JUMP_TARGET, BlockPos::new(8, 300, 0));
        body.set_brain(brain);
        body.tick_brain(breeze.as_ref());

        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(entity.pose.load(), EntityPose::Inhaling);
        assert!(entity.velocity.load().y > 0.0);
        assert!(body.living_entity.discard_friction.load(Relaxed));

        let falling_velocity = Vector3::new(0.0, -0.1, 0.0);
        entity.on_ground.store(false, Relaxed);
        entity.velocity.store(falling_velocity);
        body.tick_brain(breeze.as_ref());
        assert_eq!(observer.attempts.load(Relaxed), 1);
        assert_eq!(entity.velocity.load(), falling_velocity);

        entity.on_ground.store(true, Relaxed);
        body.tick_brain(breeze.as_ref());
        assert_eq!(entity.pose.load(), EntityPose::Standing);
        assert!(!body.living_entity.discard_friction.load(Relaxed));
        assert!(body.brain_has_memory_value(types::BREEZE_SHOOT.id()));
        server.stop();
        server.shutdown().await;
        Ok(())
    }
}
