use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::super::super::BrainTick;
use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::walk_target::WalkTarget;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use crate::entity::EntityBase;

const NO_TIMEOUT: i32 = i32::MAX / 2;
const FOLLOW_DISTANCE_SQR: f64 = 16.0;
const CLOSE_ENOUGH_DISTANCE: i32 = 2;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
];

pub struct LookAndFollowTradingPlayerSink {
    speed_modifier: f32,
}

impl LookAndFollowTradingPlayerSink {
    #[must_use]
    pub const fn new(speed_modifier: f32) -> Self {
        Self { speed_modifier }
    }

    fn follow_player(&self, tick: &mut BrainTick<'_>) {
        let Some(player) = tick.mob.get_trading_player() else {
            return;
        };
        let target: Arc<dyn EntityBase> = player;
        tick.state.set(
            types::WALK_TARGET,
            WalkTarget::new(
                Arc::new(EntityTracker::new(target.clone(), false)),
                self.speed_modifier,
                CLOSE_ENOUGH_DISTANCE,
            ),
        );
        tick.state.set(
            types::LOOK_TARGET,
            Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
        );
    }
}

impl Behavior for LookAndFollowTradingPlayerSink {
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
        let Some(player) = tick.mob.get_trading_player() else {
            return false;
        };
        let entity = tick.mob.get_entity();
        entity.is_alive()
            && !entity.is_in_water()
            && tick
                .mob
                .get_mob_entity()
                .living_entity
                .hurt_cooldown
                .load(Relaxed)
                == 0
            && entity
                .pos
                .load()
                .squared_distance_to_vec(&player.get_entity().pos.load())
                <= FOLLOW_DISTANCE_SQR
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.check_extra_start_conditions(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.follow_player(tick);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        self.follow_player(tick);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "LookAndFollowTradingPlayerSink"
    }
}
