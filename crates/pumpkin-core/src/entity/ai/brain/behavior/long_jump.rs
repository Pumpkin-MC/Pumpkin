use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use rand::seq::SliceRandom;

use super::super::BrainTick;
use super::super::memory::{
    MemoryModuleId, MemoryStatus,
    position_tracker::{BlockPosTracker, PositionTracker},
    types,
};
use super::Behavior;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;

const LONG_JUMP_TIME_OUT_DURATION: i32 = 100;
const FIND_JUMP_TRIES: i32 = 20;
const PREPARE_JUMP_DURATION: i64 = 40;
const MIN_PATHFIND_DISTANCE_TO_VALID_JUMP: f32 = 8.0;
const TIME_OUT_DURATION: i32 = 200;
const ALLOWED_ANGLES: [i32; 4] = [65, 70, 75, 80];

/// Inclusive range sampled like `UniformInt`.
pub type TimeBetweenLongJumps = (i32, i32);

fn sample(tick: &BrainTick<'_>, range: TimeBetweenLongJumps) -> i32 {
    tick.mob.get_random().random_range(range.0..=range.1)
}

/// Keeps the mob in its jumping pose until it lands, then starts the cooldown.
pub struct LongJumpMidJump {
    time_between_long_jumps: TimeBetweenLongJumps,
    landing_sound: Sound,
}

impl LongJumpMidJump {
    #[must_use]
    pub const fn new(time_between_long_jumps: TimeBetweenLongJumps, landing_sound: Sound) -> Self {
        Self {
            time_between_long_jumps,
            landing_sound,
        }
    }
}

impl Behavior for LongJumpMidJump {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::LONG_JUMP_MID_JUMP.id(), MemoryStatus::ValuePresent),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        LONG_JUMP_TIME_OUT_DURATION
    }

    fn max_duration(&self) -> i32 {
        LONG_JUMP_TIME_OUT_DURATION
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        !tick.mob.get_entity().on_ground.load(Relaxed)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.mob
            .get_mob_entity()
            .living_entity
            .discard_friction
            .store(true, Relaxed);
        tick.set_pose(EntityPose::LongJumping);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        let entity = tick.mob.get_entity();
        let landing = entity.on_ground.load(Relaxed).then(|| {
            let velocity = entity.velocity.load();
            entity.velocity.store(Vector3::new(
                velocity.x * f64::from(0.1f32),
                velocity.y,
                velocity.z * f64::from(0.1f32),
            ));
            (self.landing_sound, entity.pos.load())
        });
        let world = tick.world.clone();
        let cooldown_range = self.time_between_long_jumps;
        tick.defer_with_update(
            move |mob| {
                let entity = mob.get_entity();
                if let Some((sound, position)) = landing {
                    // Both sends can synchronously call packet listeners.
                    entity.send_velocity();
                    world.play_sound_fine(sound, SoundCategory::Neutral, &position, 2.0, 1.0);
                }
                let living = &mob.get_mob_entity().living_entity;
                living.discard_friction.store(false, Relaxed);
                if !entity.is_removed() && Arc::ptr_eq(&entity.world.load(), &world) {
                    entity.set_pose(EntityPose::Standing);
                }
            },
            move |tick, ()| {
                tick.state.erase(types::LONG_JUMP_MID_JUMP.id());
                let cooldown = sample(tick, cooldown_range);
                tick.state.set(types::LONG_JUMP_COOLDOWN_TICKS, cooldown);
            },
        );
    }

    fn debug_name(&self) -> &'static str {
        "LongJumpMidJump"
    }
}

struct PossibleJump {
    target_pos: BlockPos,
    weight: i32,
}

/// Prefers landing on `tag` with chance `chance`, falling back to any other candidate.
pub struct PreferredBlock {
    pub tag: &'static tag::Tag,
    pub chance: f32,
}

/// Jumps to a random reachable spot the mob could not simply walk to.
pub struct LongJumpToRandomPos {
    time_between_long_jumps: TimeBetweenLongJumps,
    max_long_jump_height: i32,
    max_long_jump_width: i32,
    max_jump_velocity_multiplier: f32,
    jump_sound: Sound,
    acceptable_landing_spot: fn(&dyn Mob, BlockPos) -> bool,
    preferred: Option<PreferredBlock>,
    jump_candidates: Vec<PossibleJump>,
    not_preferred_jump_candidates: Vec<PossibleJump>,
    currently_wanting_preferred_ones: bool,
    initial_position: Option<Vector3<f64>>,
    chosen_jump: Option<Vector3<f64>>,
    find_jump_tries: i32,
    prepare_jump_start: i64,
}

impl LongJumpToRandomPos {
    #[must_use]
    pub const fn new(
        time_between_long_jumps: TimeBetweenLongJumps,
        max_long_jump_height: i32,
        max_long_jump_width: i32,
        max_jump_velocity_multiplier: f32,
        jump_sound: Sound,
        acceptable_landing_spot: fn(&dyn Mob, BlockPos) -> bool,
        preferred: Option<PreferredBlock>,
    ) -> Self {
        Self {
            time_between_long_jumps,
            max_long_jump_height,
            max_long_jump_width,
            max_jump_velocity_multiplier,
            jump_sound,
            acceptable_landing_spot,
            preferred,
            jump_candidates: Vec::new(),
            not_preferred_jump_candidates: Vec::new(),
            currently_wanting_preferred_ones: false,
            initial_position: None,
            chosen_jump: None,
            find_jump_tries: 0,
            prepare_jump_start: 0,
        }
    }

    fn take_random_candidate(&mut self, tick: &BrainTick<'_>) -> Option<PossibleJump> {
        let total_weight: i32 = self.jump_candidates.iter().map(|jump| jump.weight).sum();
        if total_weight <= 0 {
            return None;
        }
        let mut selection = tick.mob.get_random().random_range(0..total_weight);
        let index = self.jump_candidates.iter().position(|jump| {
            selection -= jump.weight;
            selection < 0
        })?;
        Some(self.jump_candidates.remove(index))
    }

    fn get_jump_candidate(&mut self, tick: &BrainTick<'_>) -> Option<PossibleJump> {
        let Some(preferred_tag) = self
            .preferred
            .as_ref()
            .filter(|_| self.currently_wanting_preferred_ones)
            .map(|preferred| preferred.tag)
        else {
            return self.take_random_candidate(tick);
        };
        while !self.jump_candidates.is_empty() {
            if let Some(jump) = self.take_random_candidate(tick) {
                if tick
                    .world
                    .get_block(&jump.target_pos.down())
                    .has_tag(preferred_tag)
                {
                    return Some(jump);
                }
                self.not_preferred_jump_candidates.push(jump);
            }
        }
        (!self.not_preferred_jump_candidates.is_empty())
            .then(|| self.not_preferred_jump_candidates.remove(0))
    }

    fn pick_candidate(&mut self, tick: &mut BrainTick<'_>) {
        while !self.jump_candidates.is_empty() {
            let Some(jump) = self.get_jump_candidate(tick) else {
                continue;
            };
            let target_pos = jump.target_pos;
            let body_pos = tick.mob.get_entity().block_pos.load();
            if (body_pos.0.x == target_pos.0.x && body_pos.0.z == target_pos.0.z)
                || !(self.acceptable_landing_spot)(tick.mob, target_pos)
            {
                continue;
            }
            let Some(jump_vector) =
                self.calculate_optimal_jump_vector(tick.mob, target_pos.to_centered_f64())
            else {
                continue;
            };
            let tracker: Arc<dyn PositionTracker> = Arc::new(BlockPosTracker::new(target_pos));
            tick.state.set(types::LOOK_TARGET, tracker);
            let living = &tick.mob.get_mob_entity().living_entity;
            let path = tick
                .mob
                .get_mob_entity()
                .steered()
                .navigator()
                .create_path_with_max_length(
                    living,
                    target_pos.0.to_f64(),
                    0,
                    MIN_PATHFIND_DISTANCE_TO_VALID_JUMP,
                );
            if path.is_none_or(|path| !path.can_reach()) {
                self.chosen_jump = Some(jump_vector);
                self.prepare_jump_start = tick.time;
                return;
            }
        }
    }

    fn calculate_optimal_jump_vector(
        &self,
        mob: &dyn Mob,
        target: Vector3<f64>,
    ) -> Option<Vector3<f64>> {
        let mut angles = ALLOWED_ANGLES;
        angles.shuffle(&mut rand::rng());
        let max_jump_velocity = (mob
            .get_mob_entity()
            .living_entity
            .get_attribute_value(&Attributes::JUMP_STRENGTH)
            * f64::from(self.max_jump_velocity_multiplier)) as f32;
        angles.into_iter().find_map(|angle| {
            calculate_jump_vector_for_angle(mob, target, max_jump_velocity, angle, true)
        })
    }

    fn set_half_cooldown(&self, tick: &mut BrainTick<'_>) {
        let cooldown = sample(tick, self.time_between_long_jumps) / 2;
        tick.state.set(types::LONG_JUMP_COOLDOWN_TICKS, cooldown);
    }
}

impl Behavior for LongJumpToRandomPos {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 3] = [
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (
                types::LONG_JUMP_COOLDOWN_TICKS.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::LONG_JUMP_MID_JUMP.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn max_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let entity = tick.mob.get_entity();
        let can_start = entity.on_ground.load(Relaxed)
            && !entity.is_in_water()
            && !entity.touching_lava.load(Relaxed)
            && tick.world.get_block(&entity.block_pos.load()) != &pumpkin_data::Block::HONEY_BLOCK;
        if !can_start {
            self.set_half_cooldown(tick);
        }
        can_start
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let entity = tick.mob.get_entity();
        let is_valid = self.initial_position == Some(entity.pos.load())
            && self.find_jump_tries > 0
            && !entity.is_in_water()
            && (self.chosen_jump.is_some() || !self.jump_candidates.is_empty());
        if !is_valid && !tick.state.has_memory_value(types::LONG_JUMP_MID_JUMP.id()) {
            self.set_half_cooldown(tick);
            tick.state.erase(types::LOOK_TARGET.id());
        }
        is_valid
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.chosen_jump = None;
        self.find_jump_tries = FIND_JUMP_TRIES;
        let entity = tick.mob.get_entity();
        self.initial_position = Some(entity.pos.load());
        let mob_pos = entity.block_pos.load();
        let (width, height) = (self.max_long_jump_width, self.max_long_jump_height);
        self.jump_candidates.clear();
        for z in mob_pos.0.z - width..=mob_pos.0.z + width {
            for y in mob_pos.0.y - height..=mob_pos.0.y + height {
                for x in mob_pos.0.x - width..=mob_pos.0.x + width {
                    let pos = BlockPos::new(x, y, z);
                    if pos == mob_pos {
                        continue;
                    }
                    let (dx, dy, dz) = (
                        f64::from(x - mob_pos.0.x),
                        f64::from(y - mob_pos.0.y),
                        f64::from(z - mob_pos.0.z),
                    );
                    self.jump_candidates.push(PossibleJump {
                        target_pos: pos,
                        weight: (dx * dx + dy * dy + dz * dz).ceil() as i32,
                    });
                }
            }
        }
        if let Some(preferred) = &self.preferred {
            self.not_preferred_jump_candidates.clear();
            self.currently_wanting_preferred_ones =
                tick.mob.get_random().random::<f32>() < preferred.chance;
        }
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(chosen_jump) = self.chosen_jump else {
            self.find_jump_tries -= 1;
            self.pick_candidate(tick);
            return;
        };
        if tick.time - self.prepare_jump_start >= PREPARE_JUMP_DURATION {
            let entity = tick.mob.get_entity();
            entity.yaw.store(entity.body_yaw.load());
            let living = &tick.mob.get_mob_entity().living_entity;
            living.discard_friction.store(true, Relaxed);
            let length = chosen_jump.length();
            let length_with_jump_boost = length + f64::from(living.get_jump_boost_power());
            entity
                .velocity
                .store(chosen_jump * (length_with_jump_boost / length));
            tick.state.set(types::LONG_JUMP_MID_JUMP, true);
            let sound = self.jump_sound;
            let position = entity.pos.load();
            let world = tick.world.clone();
            tick.defer(move |mob| {
                mob.get_entity().send_velocity();
                world.play_sound_fine(sound, SoundCategory::Neutral, &position, 1.0, 1.0);
            });
        }
    }

    fn debug_name(&self) -> &'static str {
        "LongJumpToRandomPos"
    }
}

/// The landing check used when a mob passes none of its own.
#[must_use]
pub fn default_acceptable_landing_spot(mob: &dyn Mob, target_pos: BlockPos) -> bool {
    let entity = mob.get_entity();
    let world = entity.world.load_full();
    if !world.get_block_state(&target_pos.down()).is_solid_render() {
        return false;
    }
    let mut context = PathfindingContext::new(entity.block_pos.load().0, world);
    let path_type = context.get_land_node_type(target_pos.0);
    mob.get_mob_entity()
        .steered()
        .navigator()
        .get_pathfinding_malus(path_type)
        == 0.0
}

/// Whether the landing cell is a trapdoor or the air above one.
#[must_use]
pub fn lands_on_trapdoor(mob: &dyn Mob, target_pos: BlockPos) -> bool {
    let entity = mob.get_entity();
    let world = entity.world.load_full();
    let target_is_air = world.get_block_state(&target_pos).is_air();
    let mut context = PathfindingContext::new(entity.block_pos.load().0, world);
    context.get_land_node_type(target_pos.0) == PathType::Trapdoor
        || (target_is_air && context.get_land_node_type(target_pos.down().0) == PathType::Trapdoor)
}

/// Launch velocity reaching `target` at `angle` degrees, if one exists below the cap.
#[must_use]
pub fn calculate_jump_vector_for_angle(
    mob: &dyn Mob,
    target: Vector3<f64>,
    max_jump_velocity: f32,
    angle: i32,
    check_collision: bool,
) -> Option<Vector3<f64>> {
    let entity = mob.get_entity();
    let mob_pos = entity.pos.load();
    let direction_plane =
        Vector3::new(target.x - mob_pos.x, 0.0, target.z - mob_pos.z).normalize() * 0.5;
    let direction = target - direction_plane - mob_pos;
    let angrad = angle as f32 * std::f32::consts::PI / 180.0;
    let xz_ang = direction.z.atan2(direction.x);
    let r2 = direction.x * direction.x + direction.z * direction.z;
    let r = r2.sqrt();
    let y = direction.y;
    let g = mob.get_gravity();
    let sin2ang = f64::from(2.0 * angrad).sin();
    let cosangsqr = f64::from(angrad).cos().powi(2);
    let sinangrad = f64::from(angrad).sin();
    let cosangrad = f64::from(angrad).cos();
    let sin_xz_ang = xz_ang.sin();
    let cos_xz_ang = xz_ang.cos();
    let v0sqr = r2 * g / (r * sin2ang - 2.0 * y * cosangsqr);
    if v0sqr < 0.0 {
        return None;
    }
    let v0 = v0sqr.sqrt();
    if v0 > f64::from(max_jump_velocity) {
        return None;
    }
    let v0r = v0 * cosangrad;
    let v0y = v0 * sinangrad;
    if check_collision {
        let samples = (r / v0r).ceil() as i32 * 2;
        let dimensions = entity.get_dimensions(EntityPose::LongJumping);
        let mut ri = 0.0;
        let mut previous: Option<Vector3<f64>> = None;
        for _ in 0..samples - 1 {
            ri += r / f64::from(samples);
            let yi =
                sinangrad / cosangrad * ri - ri.powi(2) * g / (2.0 * v0sqr * cosangrad.powi(2));
            let sample = Vector3::new(
                mob_pos.x + ri * cos_xz_ang,
                mob_pos.y + yi,
                mob_pos.z + ri * sin_xz_ang,
            );
            if let Some(previous) = previous
                && !is_clear_transition(mob, &dimensions, previous, sample)
            {
                return None;
            }
            previous = Some(sample);
        }
    }
    Some(Vector3::new(v0r * cos_xz_ang, v0y, v0r * sin_xz_ang) * f64::from(0.95f32))
}

fn is_clear_transition(
    mob: &dyn Mob,
    dimensions: &pumpkin_util::math::boundingbox::EntityDimensions,
    from: Vector3<f64>,
    to: Vector3<f64>,
) -> bool {
    let direction = to - from;
    let min_dimension = f64::from(dimensions.width.min(dimensions.height));
    let checks = (direction.length() / min_dimension).ceil() as i32;
    let step = direction.normalize() * (min_dimension * f64::from(0.9f32));
    let world = mob.get_entity().world.load();
    let mut next = from;
    for i in 0..checks {
        next = if i == checks - 1 { to } else { next + step };
        let bounding_box = BoundingBox::new_from_pos(next.x, next.y, next.z, dimensions);
        if !world.no_collision(mob, bounding_box) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, mpsc};
    use std::time::Duration;

    use pumpkin_data::packet::clientbound::play::{SET_ENTITY_MOTION, SOUND};
    use pumpkin_data::{dimension::Dimension, entity::EntityType, environment_attribute::Activity};

    use super::*;
    use crate::entity::ai::test_support::{java_sound_recipient, server};
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{
        Entity, EntityBase,
        ai::brain::{Brain, activity::ActivityData, behavior::Timed},
    };
    use crate::plugin::{
        BoxFuture, EventRwHandler, api::events::EventPriority, server::packet::PacketSentEvent,
    };
    use crate::server::Server;

    struct LandingPackets {
        mob: Arc<CowEntity>,
        observations: Mutex<Vec<(i32, bool, bool, EntityPose)>>,
    }

    impl EventRwHandler<PacketSentEvent> for LandingPackets {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id != SET_ENTITY_MOTION.0 && event.packet_id != SOUND.0 {
                    return;
                }
                let (send, receive) = mpsc::channel();
                let mob = self.mob.clone();
                std::thread::spawn(move || {
                    let mid_jump = mob.mob_entity.with_brain(mob.as_ref(), |brain| {
                        brain.get(types::LONG_JUMP_MID_JUMP) == Some(&true)
                    });
                    let _ = send.send(mid_jump);
                });
                let mid_jump = receive.recv_timeout(Duration::from_secs(3));
                let friction_discarded = self
                    .mob
                    .mob_entity
                    .living_entity
                    .discard_friction
                    .load(Relaxed);
                self.observations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((
                        event.packet_id,
                        mid_jump == Ok(true),
                        friction_discarded,
                        self.mob.get_entity().pose.load(),
                    ));
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn landing_packets_allow_brain_reads_before_cleanup()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (player, _peer) = java_sound_recipient(&server, &world, "LandingTest")?;

        let mob = CowEntity::new(Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::COW,
        ));
        let entity = mob.get_entity();
        let chunk = entity.chunk_pos.load();
        player
            .watched_section
            .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
                chunk,
                std::num::NonZeroU8::new(8).ok_or("Expected nonzero view distance")?,
            ));
        player
            .chunk_sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .mark_sent_out_of_band(chunk);
        world.add_entity_silent(mob.clone());
        assert!(world.is_tracked_by_any_player(entity));
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(Timed::new(LongJumpMidJump::new(
                (40, 40),
                Sound::EntityGoatStep,
            )))],
        ));
        brain.register_memory(types::LONG_JUMP_COOLDOWN_TICKS.id());
        brain.set(types::LONG_JUMP_MID_JUMP, true);
        mob.mob_entity.set_brain(brain);
        entity.on_ground.store(false, Relaxed);
        mob.mob_entity.tick_brain(mob.as_ref());
        assert_eq!(entity.pose.load(), EntityPose::LongJumping);

        let listener = Arc::new(LandingPackets {
            mob: mob.clone(),
            observations: Mutex::new(Vec::new()),
        });
        server
            .plugin_manager
            .register_blocking(listener.clone(), EventPriority::Normal, None);
        entity.velocity.store(Vector3::new(1.0, 0.0, 0.0));
        entity.on_ground.store(true, Relaxed);
        mob.mob_entity.tick_brain(mob.as_ref());

        assert_eq!(
            *listener
                .observations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            [
                (SET_ENTITY_MOTION.0, true, true, EntityPose::LongJumping),
                (SOUND.0, true, true, EntityPose::LongJumping)
            ],
        );
        assert_eq!(entity.pose.load(), EntityPose::Standing);
        assert!(!mob.mob_entity.living_entity.discard_friction.load(Relaxed));
        mob.mob_entity.with_brain(mob.as_ref(), |brain| {
            assert!(brain.get(types::LONG_JUMP_MID_JUMP).is_none());
            assert_eq!(brain.get(types::LONG_JUMP_COOLDOWN_TICKS), Some(&40));
        });
        server.shutdown().await;
        Ok(())
    }
}
