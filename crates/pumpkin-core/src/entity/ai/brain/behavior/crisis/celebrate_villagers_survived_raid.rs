use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus};
use super::super::timed::Behavior;
use super::move_to_sky_seeing_spot::has_no_blocks_above;

pub struct CelebrateVillagersSurvivedRaid {
    min_duration: i32,
    max_duration: i32,
    current_raid: Option<i32>,
}

impl CelebrateVillagersSurvivedRaid {
    #[must_use]
    pub const fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            min_duration,
            max_duration,
            current_raid: None,
        }
    }
}

impl Behavior for CelebrateVillagersSurvivedRaid {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &[]
    }

    fn min_duration(&self) -> i32 {
        self.min_duration
    }

    fn max_duration(&self) -> i32 {
        self.max_duration
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let test_pos = tick.mob.get_entity().block_pos.load();
        let raid = {
            let raids = tick
                .world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids
                .get_raid_at(&test_pos)
                .map(|raid| (raid.id, raid.is_victory()))
        };
        self.current_raid = raid.map(|(id, _)| id);
        raid.is_some_and(|(_, victory)| victory)
            && has_no_blocks_above(tick.world, tick.mob.get_entity().pos.load().y, &test_pos)
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(raid_id) = self.current_raid else {
            return false;
        };
        let raids = tick
            .world
            .raids
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        raids.get(raid_id).is_some_and(|raid| !raid.is_stopped())
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        self.current_raid = None;
        let pos = tick.mob.get_entity().block_pos.load();
        tick.state
            .update_activity_from_schedule(tick.world, tick.time, &pos);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let mut random = tick.mob.get_random();
        let _ = random.random_range(0..100) == 0;
        let _ = random.random_range(0..200) == 0
            && has_no_blocks_above(
                tick.world,
                tick.mob.get_entity().pos.load().y,
                &tick.mob.get_entity().block_pos.load(),
            );
    }

    fn debug_name(&self) -> &'static str {
        "CelebrateVillagersSurvivedRaid"
    }
}
