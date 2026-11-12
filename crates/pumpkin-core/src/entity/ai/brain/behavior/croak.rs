use pumpkin_data::entity::EntityPose;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::Behavior;

const CROAK_TICKS: i32 = 60;
const TIME_OUT_DURATION: i32 = 100;

#[derive(Default)]
pub struct Croak {
    croak_counter: i32,
}

impl Behavior for Croak {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 1] =
            [(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn max_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_entity().pose.load() == EntityPose::Standing
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let entity = tick.mob.get_entity();
        if !entity.is_in_water()
            && !entity
                .touching_lava
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            entity.set_pose(EntityPose::Croaking);
            self.croak_counter = 0;
        }
    }

    fn tick(&mut self, _tick: &mut BrainTick<'_>) {
        self.croak_counter += 1;
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.mob.get_entity().set_pose(EntityPose::Standing);
    }

    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        self.croak_counter < CROAK_TICKS
    }

    fn debug_name(&self) -> &'static str {
        "Croak"
    }
}
