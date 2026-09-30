use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::util::default_random_pos;

use super::super::BrainTick;
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;

const MAX_COOLDOWN_BEFORE_RETRYING: i32 = 40;

#[must_use]
pub const fn dist_manhattan(a: &BlockPos, b: &BlockPos) -> i32 {
    (a.0.x - b.0.x).abs() + (a.0.y - b.0.y).abs() + (a.0.z - b.0.z).abs()
}

#[must_use]
fn reached_target(body_pos: &BlockPos, walk_target: &WalkTarget) -> bool {
    dist_manhattan(&walk_target.target().current_block_position(), body_pos)
        <= walk_target.close_enough_dist()
}

pub struct MoveToTargetSink {
    conditions: [(MemoryModuleId, MemoryStatus); 2],
    min_timeout: i32,
    max_timeout: i32,
    remaining_cooldown: i32,
    path: Option<Path>,
    last_target_pos: Option<BlockPos>,
    speed_modifier: f32,
}

impl Default for MoveToTargetSink {
    fn default() -> Self {
        Self::new(150, 250)
    }
}

impl MoveToTargetSink {
    #[must_use]
    pub const fn new(min_timeout: i32, max_timeout: i32) -> Self {
        Self {
            conditions: [
                (types::PATH.id(), MemoryStatus::ValueAbsent),
                (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
            ],
            min_timeout,
            max_timeout,
            remaining_cooldown: 0,
            path: None,
            last_target_pos: None,
            speed_modifier: 0.0,
        }
    }

    fn create_path(tick: &BrainTick<'_>, destination: Vector3<f64>) -> Option<Path> {
        let mob_entity = tick.mob.get_mob_entity();
        let mut navigator = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.create_path(&mob_entity.living_entity, destination, 0)
    }

    fn try_compute_path(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some((target_pos, speed_modifier, reached)) =
            tick.brain.get(types::WALK_TARGET).map(|walk_target| {
                (
                    walk_target.target().current_block_position(),
                    walk_target.speed_modifier(),
                    reached_target(&tick.mob.get_entity().block_pos.load(), walk_target),
                )
            })
        else {
            return false;
        };

        let destination = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(target_pos.0.y),
            f64::from(target_pos.0.z) + 0.5,
        );
        self.path = Self::create_path(tick, destination);
        self.speed_modifier = speed_modifier;

        if reached {
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

        let partial_step = default_random_pos::get_pos_towards(
            tick.mob,
            10,
            7,
            destination,
            std::f64::consts::FRAC_PI_2,
        );
        let Some(partial_step) = partial_step else {
            return false;
        };
        self.path = Self::create_path(tick, partial_step);
        self.path.is_some()
    }

    fn move_along_path(&self, tick: &mut BrainTick<'_>) {
        let Some(path) = self.path.as_ref().map(Path::copy) else {
            return;
        };
        tick.brain.set(types::PATH, path.copy());

        let mob_entity = tick.mob.get_mob_entity();
        mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .move_to_path(
                Some(path),
                f64::from(self.speed_modifier),
                &mob_entity.living_entity,
            );
    }
}

impl Behavior for MoveToTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min_timeout
    }

    fn max_duration(&self) -> i32 {
        self.max_timeout
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if self.remaining_cooldown > 0 {
            self.remaining_cooldown -= 1;
            return false;
        }
        let Some(reached) = tick.brain.get(types::WALK_TARGET).map(|walk_target| {
            reached_target(&tick.mob.get_entity().block_pos.load(), walk_target)
        }) else {
            return false;
        };
        if !reached && self.try_compute_path(tick) {
            self.last_target_pos = tick
                .brain
                .get(types::WALK_TARGET)
                .map(|walk_target| walk_target.target().current_block_position());
            return true;
        }
        tick.brain.erase(types::WALK_TARGET.id());
        if reached {
            tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        }
        false
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.move_along_path(tick);
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        if self.path.is_none() || self.last_target_pos.is_none() {
            return false;
        }
        let Some(walk_target) = tick.brain.get(types::WALK_TARGET) else {
            return false;
        };
        if walk_target.target().tracks_spectator() {
            return false;
        }
        let navigation_done = tick
            .mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_done();
        !navigation_done && !reached_target(&tick.mob.get_entity().block_pos.load(), walk_target)
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let navigator_path = tick
            .mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_path()
            .map(Path::copy);
        let changed = match (self.path.as_ref(), navigator_path.as_ref()) {
            (None, None) => false,
            (Some(current), Some(other)) => !current.same_as_path(other),
            _ => true,
        };
        if changed {
            self.path = navigator_path;
            match &self.path {
                Some(path) => tick.brain.set(types::PATH, path.copy()),
                None => tick.brain.erase(types::PATH.id()),
            }
        }

        if self.path.is_none() {
            return;
        }
        let Some(last_target_pos) = self.last_target_pos else {
            return;
        };
        let Some(target_pos) = tick
            .brain
            .get(types::WALK_TARGET)
            .map(|walk_target| walk_target.target().current_block_position())
        else {
            return;
        };
        if target_pos.squared_distance(&last_target_pos) > 4 && self.try_compute_path(tick) {
            self.last_target_pos = Some(target_pos);
            self.move_along_path(tick);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        let still_trying = tick
            .brain
            .get(types::WALK_TARGET)
            .is_some_and(|walk_target| {
                !reached_target(&tick.mob.get_entity().block_pos.load(), walk_target)
            });
        let stuck = {
            let mob_entity = tick.mob.get_mob_entity();
            let mut navigator = mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let stuck = navigator.is_stuck();
            navigator.stop();
            stuck
        };
        if still_trying && stuck {
            self.remaining_cooldown = tick
                .mob
                .get_random()
                .random_range(0..MAX_COOLDOWN_BEFORE_RETRYING);
        }

        tick.brain.erase(types::WALK_TARGET.id());
        tick.brain.erase(types::PATH.id());
        self.path = None;
    }

    fn debug_name(&self) -> &'static str {
        "MoveToTargetSink"
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;

    use super::dist_manhattan;

    #[test]
    fn manhattan_distance_adds_every_axis() {
        assert_eq!(
            dist_manhattan(&BlockPos::new(0, 0, 0), &BlockPos::new(1, 2, 3)),
            6
        );
        assert_eq!(
            dist_manhattan(&BlockPos::new(-1, -2, -3), &BlockPos::new(0, 0, 0)),
            6
        );
    }

    #[test]
    fn a_diagonal_step_is_two_manhattan_blocks_not_one() {
        assert_eq!(
            dist_manhattan(&BlockPos::new(0, 0, 0), &BlockPos::new(1, 0, 1)),
            2
        );
    }

    #[test]
    fn manhattan_distance_is_symmetric() {
        let a = BlockPos::new(4, -7, 2);
        let b = BlockPos::new(-3, 1, 9);
        assert_eq!(dist_manhattan(&a, &b), dist_manhattan(&b, &a));
    }
}
