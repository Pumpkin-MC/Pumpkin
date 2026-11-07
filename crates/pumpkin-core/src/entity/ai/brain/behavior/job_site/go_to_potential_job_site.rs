use pumpkin_data::environment_attribute::Activity;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use super::super::utils;
use super::super::utils::world_for_dimension;

const TICKS_UNTIL_TIMEOUT: i32 = 1200;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 1] =
    [(types::POTENTIAL_JOB_SITE.id(), MemoryStatus::ValuePresent)];

pub struct GoToPotentialJobSite {
    speed_modifier: f32,
}

impl GoToPotentialJobSite {
    #[must_use]
    pub const fn new(speed_modifier: f32) -> Self {
        Self { speed_modifier }
    }
}

impl Behavior for GoToPotentialJobSite {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        TICKS_UNTIL_TIMEOUT
    }

    fn max_duration(&self) -> i32 {
        TICKS_UNTIL_TIMEOUT
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state
            .get_active_non_core_activity()
            .is_none_or(|activity| {
                matches!(activity, Activity::Idle | Activity::Work | Activity::Play)
            })
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::POTENTIAL_JOB_SITE.id())
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(global_pos) = tick.state.get(types::POTENTIAL_JOB_SITE).copied() else {
            return;
        };
        utils::set_walk_and_look_target_memories_to_block(
            tick.state,
            global_pos.pos,
            self.speed_modifier,
            1,
        );
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(global_pos) = tick.state.get(types::POTENTIAL_JOB_SITE).copied()
            && let Some(poi_level) = world_for_dimension(tick.world, global_pos.dimension)
        {
            let mut pois = poi_level
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if pois.exists(&global_pos.pos, &|_| true) {
                pois.release(&global_pos.pos);
            }
        }
        tick.state.erase(types::POTENTIAL_JOB_SITE.id());
    }

    fn debug_name(&self) -> &'static str {
        "GoToPotentialJobSite"
    }
}
