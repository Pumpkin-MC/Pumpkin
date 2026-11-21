use std::sync::atomic::Ordering::Relaxed;

use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::timed::Behavior;

pub struct Swim {
    chance: f32,
}

impl Swim {
    #[must_use]
    pub const fn new(chance: f32) -> Self {
        Self { chance }
    }
}

impl Behavior for Swim {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &[]
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let living = &tick.mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        (entity.is_in_water() && entity.water_height.load() > living.get_swim_height())
            || entity.touching_lava.load(Relaxed)
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.check_extra_start_conditions(tick)
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if tick.mob.get_random().random::<f32>() < self.chance {
            tick.mob
                .get_mob_entity()
                .jump_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .jump();
        }
    }

    fn debug_name(&self) -> &'static str {
        "Swim"
    }
}
