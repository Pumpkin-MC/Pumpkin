use pumpkin_data::block_properties::WhiteBedLikeProperties as BedProperties;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_data::tag::{Block as BlockTag, Taggable};

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::door::close_doors_that_i_have_opened_or_passed_through;
use super::super::timed::Behavior;
use crate::entity::passive::villager::VillagerEntity;

pub const COOLDOWN_AFTER_BEING_WOKEN: i32 = 100;

const NO_TIMEOUT: i32 = i32::MAX / 2;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 5] = [
    (types::HOME.id(), MemoryStatus::ValuePresent),
    (types::LAST_WOKEN.id(), MemoryStatus::Registered),
    (types::LAST_SLEPT.id(), MemoryStatus::Registered),
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (
        types::CANT_REACH_WALK_TARGET_SINCE.id(),
        MemoryStatus::Registered,
    ),
];

#[derive(Default)]
pub struct SleepInBed {
    next_ok_start_time: i64,
}

impl SleepInBed {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_ok_start_time: 0,
        }
    }
}

impl Behavior for SleepInBed {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn max_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if tick.mob.is_passenger() {
            return false;
        }
        let Some(target) = tick.state.get(types::HOME).copied() else {
            return false;
        };
        if tick.world.dimension.id != target.dimension.id {
            return false;
        }
        if let Some(last_woken) = tick.state.get(types::LAST_WOKEN).copied()
            && recently_woken(tick.time - last_woken)
        {
            return false;
        }
        let (block, state) = tick.world.get_block_and_state(&target.pos);
        let body_pos = tick.mob.get_entity().pos.load();
        target
            .pos
            .dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
            < 2.0 * 2.0
            && block.has_tag(&BlockTag::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED)
            && !BedProperties::from_state_id(state.id).occupied
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(target) = tick.state.get(types::HOME).copied() else {
            return false;
        };
        let body_pos = tick.mob.get_entity().pos.load();
        tick.state.is_active(Activity::Rest)
            && body_pos.y > f64::from(target.pos.0.y) + 0.4
            && target
                .pos
                .dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
                < 1.14 * 1.14
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if tick.time > self.next_ok_start_time {
            if tick.state.has_memory_value(types::DOORS_TO_CLOSE.id()) {
                let mut doors = tick
                    .state
                    .get(types::DOORS_TO_CLOSE)
                    .cloned()
                    .unwrap_or_default();
                let nearest_entities = tick
                    .state
                    .get(types::NEAREST_LIVING_ENTITIES)
                    .map(Vec::as_slice);
                close_doors_that_i_have_opened_or_passed_through(
                    tick.world,
                    tick.mob,
                    None,
                    None,
                    &mut doors,
                    nearest_entities,
                );
                tick.state.set_optional(types::DOORS_TO_CLOSE, Some(doors));
            }

            if let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>()
                && let Some(home) = tick.state.get(types::HOME).copied()
                && villager.start_sleeping(home.pos)
            {
                tick.state.set(types::LAST_SLEPT, tick.time);
            }

            tick.state.erase(types::WALK_TARGET.id());
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>()
            && villager.is_sleeping()
        {
            villager.stop_sleeping();
            tick.state.set(types::LAST_WOKEN, tick.time);
            self.next_ok_start_time = tick.time + 40;
        }
    }

    fn debug_name(&self) -> &'static str {
        "SleepInBed"
    }
}

fn recently_woken(time_since_last_woken: i64) -> bool {
    time_since_last_woken > 0 && time_since_last_woken < i64::from(COOLDOWN_AFTER_BEING_WOKEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recently_woken_is_strictly_inside_the_cooldown_window() {
        assert!(!recently_woken(-1));
        assert!(!recently_woken(0));
        assert!(recently_woken(1));
        assert!(recently_woken(99));
        assert!(!recently_woken(100));
        assert!(!recently_woken(101));
    }
}
