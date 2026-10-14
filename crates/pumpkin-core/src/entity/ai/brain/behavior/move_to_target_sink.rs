use std::sync::PoisonError;

use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::pathfinder::{NavigatorGoal, path::Path};
use crate::entity::ai::util::default_random_pos;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, WalkTarget, types};
use super::Behavior;

const MAX_COOLDOWN_BEFORE_RETRYING: i32 = 40;
const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 3] = [
    (
        types::CANT_REACH_WALK_TARGET_SINCE.id(),
        MemoryStatus::Registered,
    ),
    (types::PATH.id(), MemoryStatus::ValueAbsent),
    (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
];

pub struct MoveToTargetSink {
    remaining_cooldown: i32,
    path: Option<Path>,
    last_target_pos: Option<BlockPos>,
    speed_modifier: f32,
}

impl Default for MoveToTargetSink {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveToTargetSink {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            remaining_cooldown: 0,
            path: None,
            last_target_pos: None,
            speed_modifier: 0.0,
        }
    }

    fn reached_target(tick: &BrainTick<'_>, walk_target: &WalkTarget) -> bool {
        walk_target
            .target()
            .current_block_position()
            .manhattan_distance(tick.mob.get_entity().block_pos.load())
            <= walk_target.close_enough_dist()
    }

    fn try_compute_path(&mut self, tick: &mut BrainTick<'_>, walk_target: &WalkTarget) -> bool {
        let target_pos = walk_target.target().current_block_position();
        let mob = tick.mob.get_mob_entity();
        self.path = mob
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .create_path(&mob.living_entity, target_pos.to_f64(), 0);
        self.speed_modifier = walk_target.speed_modifier();

        if Self::reached_target(tick, walk_target) {
            tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
            return false;
        }
        if self.path.as_ref().is_some_and(Path::can_reach) {
            tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        } else if !tick
            .brain
            .has_memory_value(types::CANT_REACH_WALK_TARGET_SINCE.id())
        {
            tick.brain
                .set(types::CANT_REACH_WALK_TARGET_SINCE, tick.time);
        }
        if self.path.is_some() {
            return true;
        }
        let target = target_pos.to_f64();
        if let Some(partial_step) = default_random_pos::get_pos_towards(
            tick.mob,
            10,
            7,
            target,
            f64::from(std::f32::consts::FRAC_PI_2),
        ) {
            self.path = mob
                .navigator
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .create_path(&mob.living_entity, partial_step, 0);
        }
        self.path.is_some()
    }
}

impl Behavior for MoveToTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        150
    }

    fn max_duration(&self) -> i32 {
        250
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if self.remaining_cooldown > 0 {
            self.remaining_cooldown -= 1;
            return false;
        }
        let Some(walk_target) = tick.brain.get(types::WALK_TARGET).cloned() else {
            return false;
        };
        let reached = Self::reached_target(tick, &walk_target);
        if !reached && self.try_compute_path(tick, &walk_target) {
            self.last_target_pos = Some(walk_target.target().current_block_position());
            return true;
        }
        tick.brain.erase(types::WALK_TARGET.id());
        if reached {
            tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        }
        false
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        self.path.is_some()
            && self.last_target_pos.is_some()
            && tick.brain.get(types::WALK_TARGET).is_some_and(|target| {
                !Self::reached_target(tick, target) && !target.target().is_spectator()
            })
            && !tick
                .mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_done()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.brain.set_optional(types::PATH, self.path.clone());
        let mob = tick.mob.get_mob_entity();
        let mut navigation = mob.navigator.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(path) = &self.path {
            // Pumpkin also needs a movement goal when installing a computed path.
            navigation.set_progress(NavigatorGoal::new(
                mob.living_entity.entity.pos.load(),
                path.get_target().to_f64(),
                f64::from(self.speed_modifier),
            ));
        }
        navigation.move_to_path(
            self.path.clone(),
            f64::from(self.speed_modifier),
            &mob.living_entity,
        );
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let path = tick
            .mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_path()
            .cloned();
        if self.path != path {
            self.path.clone_from(&path);
            tick.brain.set_optional(types::PATH, path.clone());
        }
        if path.is_some()
            && let Some(last_target_pos) = self.last_target_pos
            && let Some(walk_target) = tick.brain.get(types::WALK_TARGET).cloned()
            && walk_target
                .target()
                .current_block_position()
                .squared_distance(&last_target_pos)
                > 4
            && self.try_compute_path(tick, &walk_target)
        {
            self.last_target_pos = Some(walk_target.target().current_block_position());
            self.start(tick);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        let mut navigation = tick
            .mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if tick
            .brain
            .get(types::WALK_TARGET)
            .is_some_and(|target| !Self::reached_target(tick, target))
            && navigation.is_stuck()
        {
            self.remaining_cooldown = tick
                .mob
                .get_random()
                .random_range(0..MAX_COOLDOWN_BEFORE_RETRYING);
        }
        navigation.stop();
        tick.brain.erase(types::WALK_TARGET.id());
        tick.brain.erase(types::PATH.id());
        self.path = None;
    }

    fn debug_name(&self) -> &'static str {
        "MoveToTargetSink"
    }
}
