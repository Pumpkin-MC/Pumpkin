use std::sync::PoisonError;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::{BlockPos, OutwardIterator};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::util::{goal_utils, land_random_pos};

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, WalkTarget, types};
use super::Behavior;

const PANIC_MIN_DURATION: i32 = 100;
const PANIC_MAX_DURATION: i32 = 120;
const PANIC_DISTANCE_HORIZONTAL: i32 = 5;
const PANIC_DISTANCE_VERTICAL: i32 = 4;
const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::IS_PANICKING.id(), MemoryStatus::Registered),
    (types::HURT_BY.id(), MemoryStatus::Registered),
];

pub struct AnimalPanic {
    speed_multiplier: f32,
}

impl AnimalPanic {
    #[must_use]
    pub const fn new(speed_multiplier: f32) -> Self {
        Self { speed_multiplier }
    }

    fn get_panic_pos(tick: &BrainTick<'_>) -> Option<Vector3<f64>> {
        if tick.mob.get_entity().is_on_fire()
            && let Some(water) = Self::look_for_water(tick)
        {
            return Some(water.to_f64());
        }
        land_random_pos::get_pos(tick.mob, PANIC_DISTANCE_HORIZONTAL, PANIC_DISTANCE_VERTICAL)
    }

    fn look_for_water(tick: &BrainTick<'_>) -> Option<BlockPos> {
        let mob = tick.mob.get_entity();
        let position = mob.block_pos.load();
        if tick
            .world
            .get_block_state(&position)
            .get_block_collision_shapes_at(&position)
            .next()
            .is_some()
        {
            return None;
        }
        let width = mob.entity_dimension.load().width.ceil() as i32;
        OutwardIterator::new(
            position,
            PANIC_DISTANCE_HORIZONTAL,
            1,
            PANIC_DISTANCE_HORIZONTAL,
        )
        .find(|candidate| {
            goal_utils::is_water(tick.world, candidate)
                && (width != 2
                    || (goal_utils::is_water(tick.world, &candidate.south())
                        && goal_utils::is_water(tick.world, &candidate.east())
                        && goal_utils::is_water(tick.world, &candidate.south().east())))
        })
    }
}

impl Behavior for AnimalPanic {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        PANIC_MIN_DURATION
    }

    fn max_duration(&self) -> i32 {
        PANIC_MAX_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.brain.get(types::HURT_BY).is_some_and(|damage| {
            damage
                .damage_type
                .has_tag(&tag::DamageType::MINECRAFT_PANIC_CAUSES)
        }) || tick.brain.has_memory_value(types::IS_PANICKING.id())
    }

    fn can_still_use(&mut self, _tick: &BrainTick<'_>) -> bool {
        true
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.brain.set(types::IS_PANICKING, true);
        tick.brain.erase(types::WALK_TARGET.id());
        tick.mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.brain.erase(types::IS_PANICKING.id());
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let done = tick
            .mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_done();
        if done && let Some(position) = Self::get_panic_pos(tick) {
            tick.brain.set(
                types::WALK_TARGET,
                WalkTarget::from_vec(position, self.speed_multiplier, 0),
            );
        }
    }

    fn debug_name(&self) -> &'static str {
        "AnimalPanic"
    }
}
