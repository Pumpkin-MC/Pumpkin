use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::position_tracker::{BlockPosTracker, PositionTracker};
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use crate::entity::passive::villager::VillagerEntity;

const CHECK_COOLDOWN: i64 = 300;
const DISTANCE: f64 = 1.73;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::JOB_SITE.id(), MemoryStatus::ValuePresent),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
];

pub struct WorkAtPoi {
    last_check: i64,
    use_workstation: Option<fn(&mut BrainTick<'_>, &VillagerEntity)>,
}

impl WorkAtPoi {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_check: 0,
            use_workstation: None,
        }
    }

    #[must_use]
    pub const fn with_workstation(
        use_workstation: fn(&mut BrainTick<'_>, &VillagerEntity),
    ) -> Self {
        Self {
            last_check: 0,
            use_workstation: Some(use_workstation),
        }
    }

    fn is_job_site_close(job_site: &BlockPos, tick: &BrainTick<'_>) -> bool {
        let pos = tick.mob.get_entity().pos.load();
        job_site.dist_to_center_sqr(pos.x, pos.y, pos.z) < DISTANCE * DISTANCE
    }
}

impl Behavior for WorkAtPoi {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if tick.time - self.last_check < CHECK_COOLDOWN {
            return false;
        }
        if tick.mob.get_random().random_range(0..2) != 0 {
            return false;
        }
        self.last_check = tick.time;
        let Some(job_site) = tick.state.get(types::JOB_SITE).copied() else {
            return false;
        };
        job_site.dimension.id == tick.world.dimension.id
            && Self::is_job_site_close(&job_site.pos, tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set(types::LAST_WORKED_AT_POI, tick.time);
        if let Some(job_site) = tick.state.get(types::JOB_SITE).copied() {
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(BlockPosTracker::new(job_site.pos)) as Arc<dyn PositionTracker>,
            );
        }
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        villager.play_work_sound();
        if let Some(use_workstation) = self.use_workstation {
            use_workstation(tick, villager);
        }
        let day = tick
            .world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .query_day();
        if villager.should_restock(tick.time, day) {
            villager.restock(tick.time);
        }
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(job_site) = tick.state.get(types::JOB_SITE).copied() else {
            return false;
        };
        job_site.dimension.id == tick.world.dimension.id
            && Self::is_job_site_close(&job_site.pos, tick)
    }

    fn debug_name(&self) -> &'static str {
        "WorkAtPoi"
    }
}

impl Default for WorkAtPoi {
    fn default() -> Self {
        Self::new()
    }
}
