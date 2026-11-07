use pumpkin_data::environment_attribute::Activity;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use crate::entity::passive::villager::VillagerEntity;

pub struct VillagerPanicTrigger;

impl VillagerPanicTrigger {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for VillagerPanicTrigger {
    fn default() -> Self {
        Self::new()
    }
}

fn is_hurt(tick: &BrainTick<'_>) -> bool {
    tick.state.has_memory_value(types::HURT_BY.id())
}

fn has_hostile(tick: &BrainTick<'_>) -> bool {
    tick.state.has_memory_value(types::NEAREST_HOSTILE.id())
}

impl Behavior for VillagerPanicTrigger {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &[]
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        is_hurt(tick) || has_hostile(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if is_hurt(tick) || has_hostile(tick) {
            if !tick.state.is_active(Activity::Panic) {
                tick.state.erase(types::PATH.id());
                tick.state.erase(types::WALK_TARGET.id());
                tick.state.erase(types::LOOK_TARGET.id());
                tick.state.erase(types::BREED_TARGET.id());
                tick.state.erase(types::INTERACTION_TARGET.id());
            }
            tick.state.set_active_activity_if_possible(Activity::Panic);
        }
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if tick.time % 100 == 0
            && let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>()
        {
            villager.spawn_golem_if_needed(tick.state, tick.world, tick.time, 3);
        }
    }

    fn debug_name(&self) -> &'static str {
        "VillagerPanicTrigger"
    }
}
