use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;

pub const TEMPTATION_COOLDOWN: i32 = 100;
pub const DEFAULT_CLOSE_ENOUGH_DIST: f64 = 2.5;
/// It never times out; it runs while the player keeps tempting.
const NO_TIMEOUT: i32 = i32::MAX / 2;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 6] = [
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (
        types::TEMPTATION_COOLDOWN_TICKS.id(),
        MemoryStatus::ValueAbsent,
    ),
    (types::TEMPTING_PLAYER.id(), MemoryStatus::ValuePresent),
    (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
];

pub struct FollowTemptation {
    speed_modifier: fn(&dyn Mob) -> f32,
    close_enough_distance: fn(&dyn Mob) -> f64,
    look_in_the_eyes: bool,
}

impl FollowTemptation {
    #[must_use]
    pub const fn new(
        speed_modifier: fn(&dyn Mob) -> f32,
        close_enough_distance: fn(&dyn Mob) -> f64,
    ) -> Self {
        Self {
            speed_modifier,
            close_enough_distance,
            look_in_the_eyes: false,
        }
    }
}

impl Behavior for FollowTemptation {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn max_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::TEMPTING_PLAYER.id())
            && !tick.state.has_memory_value(types::BREED_TARGET.id())
            && !tick.state.has_memory_value(types::IS_PANICKING.id())
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(player) = tick.state.get(types::TEMPTING_PLAYER).cloned() else {
            return;
        };
        let player: Arc<dyn EntityBase> = player;
        tick.state.set(
            types::LOOK_TARGET,
            Arc::new(EntityTracker::new(player.clone(), true)) as Arc<dyn PositionTracker>,
        );
        let close_enough = (self.close_enough_distance)(tick.mob);
        let distance = tick
            .mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&player.get_entity().pos.load());
        if distance < close_enough * close_enough {
            tick.state.erase(types::WALK_TARGET.id());
        } else {
            let tracker = EntityTracker::with_target_eye_height(
                player,
                self.look_in_the_eyes,
                self.look_in_the_eyes,
            );
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(Arc::new(tracker), (self.speed_modifier)(tick.mob), 2),
            );
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state
            .set(types::TEMPTATION_COOLDOWN_TICKS, TEMPTATION_COOLDOWN);
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "FollowTemptation"
    }
}
