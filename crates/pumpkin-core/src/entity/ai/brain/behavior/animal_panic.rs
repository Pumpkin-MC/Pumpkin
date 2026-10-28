use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget};
use super::timed::Behavior;
use crate::entity::ai::util::{goal_utils, land_random_pos};

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::IS_PANICKING.id(), MemoryStatus::Registered),
    (types::HURT_BY.id(), MemoryStatus::Registered),
];

pub struct AnimalPanic {
    speed_multiplier: f32,
}

impl AnimalPanic {
    const PANIC_MIN_DURATION: i32 = 100;
    const PANIC_MAX_DURATION: i32 = 120;
    const PANIC_DISTANCE_HORIZONTAL: i32 = 5;
    const PANIC_DISTANCE_VERTICAL: i32 = 4;

    #[must_use]
    pub const fn new(speed_multiplier: f32) -> Self {
        Self { speed_multiplier }
    }

    fn look_for_water(tick: &BrainTick<'_>) -> Option<Vector3<f64>> {
        let entity = tick.mob.get_entity();
        let origin = entity.block_pos.load();
        if !tick
            .world
            .get_block_state(&origin)
            .collision_shapes
            .is_empty()
        {
            return None;
        }
        let wide = entity.width().ceil() == 2.0;
        let is_water = |pos: &BlockPos| goal_utils::is_water(tick.world, pos);
        BlockPos::iterate_outwards(
            origin,
            Self::PANIC_DISTANCE_HORIZONTAL,
            1,
            Self::PANIC_DISTANCE_HORIZONTAL,
        )
        .find(|pos| {
            is_water(pos)
                && (!wide
                    || (is_water(&pos.south())
                        && is_water(&pos.east())
                        && is_water(&pos.south().east())))
        })
        .map(|pos| pos.to_centered_f64() - Vector3::new(0.0, 0.5, 0.0))
    }
}

impl Behavior for AnimalPanic {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        Self::PANIC_MIN_DURATION
    }

    fn max_duration(&self) -> i32 {
        Self::PANIC_MAX_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.get(types::HURT_BY).is_some_and(|damage| {
            damage
                .damage_type
                .has_tag(&tag::DamageType::MINECRAFT_PANIC_CAUSES)
        }) || tick.state.has_memory_value(types::IS_PANICKING.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set(types::IS_PANICKING, true);
        tick.state.erase(types::WALK_TARGET.id());
        tick.mob.get_mob_entity().steered().navigator().stop();
    }

    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        true
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::IS_PANICKING.id());
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if !tick.mob.is_navigator_idle() {
            return;
        }
        let pos = (tick.mob.get_entity().fire_ticks.load(Relaxed) > 0)
            .then(|| Self::look_for_water(tick))
            .flatten()
            .or_else(|| {
                land_random_pos::get_pos(
                    tick.mob,
                    Self::PANIC_DISTANCE_HORIZONTAL,
                    Self::PANIC_DISTANCE_VERTICAL,
                )
            });
        if let Some(pos) = pos {
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::from_vec(pos, self.speed_multiplier, 0),
            );
        }
    }

    fn debug_name(&self) -> &'static str {
        "AnimalPanic"
    }
}
