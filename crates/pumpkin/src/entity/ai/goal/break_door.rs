use std::sync::atomic::Ordering::Relaxed;

use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use super::interact_with_door::InteractWithDoorGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Vanilla: `BreakDoorGoal` (via `Vindicator.VindicatorBreakDoorGoal`).
///
/// Ports `net.minecraft.world.entity.ai.goal.BreakDoorGoal`: shares `DoorInteractGoal`'s door
/// discovery, but breaks the door open over time (destroying the block) rather than opening it,
/// gated on the `MOB_GRIEFING` gamerule and a caller-supplied valid-difficulty predicate. Note
/// vanilla's `getDoorBreakTime` is `Math.max(240, doorBreakTime)`, so the constructor's `seconds`
/// parameter (Vindicator passes `6`) is actually irrelevant in practice -- break time is always
/// 240 ticks (12 seconds).
///
/// The base `BreakDoorGoal.canUse`/`canContinueToUse` (`BreakDoorGoal.java:30-53`) has no raid
/// check at all -- that's added only by Vindicator's private `VindicatorBreakDoorGoal` inner
/// class, and only as an extra `canContinueToUse` clause (`Vindicator.java:188-192`:
/// `vindicator.hasActiveRaid() && super.canContinueToUse()`), not `canUse`. `raid_gated` opts a
/// caller into that same extra clause; other users (e.g. `Zombie`, whose `DOOR_BREAKING_PREDICATE`
/// is plain `d == Difficulty.HARD` with no raid involvement) leave it off.
pub struct BreakDoorGoal {
    only_during_raid: bool,
    door_pos: Option<BlockPos>,
    break_time: i32,
    last_break_progress: i32,
    valid_difficulties: fn(Difficulty) -> bool,
}

impl BreakDoorGoal {
    const DOOR_BREAK_TIME: i32 = 240;

    #[must_use]
    pub const fn new(valid_difficulties: fn(Difficulty) -> bool) -> Self {
        Self {
            only_during_raid: false,
            door_pos: None,
            break_time: 0,
            last_break_progress: -1,
            valid_difficulties,
        }
    }

    /// Vanilla: `Vindicator.VindicatorBreakDoorGoal.canContinueToUse` -- gates continued use on
    /// `hasActiveRaid()`. See the struct doc for why this is a builder rather than baked in.
    #[must_use]
    pub const fn raid_gated(mut self, only_during_raid: bool) -> Self {
        self.only_during_raid = only_during_raid;
        self
    }

    fn is_valid_difficulty(&self, difficulty: Difficulty) -> bool {
        (self.valid_difficulties)(difficulty)
    }
}

impl Goal for BreakDoorGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;

            if !entity.horizontal_collision.load(Relaxed) {
                return false;
            }

            let world = entity.world.load_full();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }
            let difficulty = world.level_info.load().difficulty;
            if !self.is_valid_difficulty(difficulty) {
                return false;
            }

            let navigator = mob_entity.navigator.lock().unwrap();
            let Some(path) = navigator.get_current_path() else {
                return false;
            };
            if path.is_done() {
                return false;
            }

            let next_index = path.get_next_node_index();
            let node_count = path.get_node_count();
            let scan_end = std::cmp::min(next_index + 2, node_count);

            for i in next_index..scan_end {
                if let Some(node) = path.get_node(i) {
                    let check_pos = BlockPos::new(node.pos.0.x, node.pos.0.y + 1, node.pos.0.z);
                    let dx = check_pos.0.x as f64 + 0.5 - entity.pos.load().x;
                    let dz = check_pos.0.z as f64 + 0.5 - entity.pos.load().z;
                    if dx * dx + dz * dz <= 2.25
                        && InteractWithDoorGoal::is_mob_interactable_door(&world, &check_pos)
                        && !InteractWithDoorGoal::is_door_open(&world, &check_pos)
                    {
                        self.door_pos = Some(check_pos);
                        return true;
                    }
                }
            }

            let mob_pos = entity.block_pos.load();
            let check_pos = BlockPos::new(mob_pos.0.x, mob_pos.0.y + 1, mob_pos.0.z);
            if InteractWithDoorGoal::is_mob_interactable_door(&world, &check_pos)
                && !InteractWithDoorGoal::is_door_open(&world, &check_pos)
            {
                self.door_pos = Some(check_pos);
                return true;
            }

            false
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(door_pos) = self.door_pos else {
                return false;
            };
            let mob_entity = mob.get_mob_entity();
            if self.only_during_raid && !mob_entity.living_entity.has_active_raid() {
                return false;
            }
            let entity = &mob_entity.living_entity.entity;
            let world = entity.world.load_full();

            if self.break_time > Self::DOOR_BREAK_TIME
                || InteractWithDoorGoal::is_door_open(&world, &door_pos)
            {
                return false;
            }
            let dist_sq = door_pos
                .to_centered_f64()
                .squared_distance_to_vec(&entity.pos.load());
            if dist_sq > 4.0 {
                return false;
            }
            self.is_valid_difficulty(world.level_info.load().difficulty)
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.break_time = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(door_pos) = self.door_pos {
                let entity = &mob.get_mob_entity().living_entity.entity;
                let world = entity.world.load_full();
                world.set_block_breaking(entity, door_pos, -1).await;
            }
        })
    }

    // Scope reduction: vanilla's 1-in-20-per-tick `levelEvent(1019, ...)` door-hit sound plus
    // arm swing is a pure client/animation effect and is not ported here.
    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(door_pos) = self.door_pos else {
                return;
            };
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();

            self.break_time += 1;
            let progress = ((self.break_time as f32 / Self::DOOR_BREAK_TIME as f32) * 10.0) as i32;
            if progress != self.last_break_progress {
                world.set_block_breaking(entity, door_pos, progress).await;
                self.last_break_progress = progress;
            }

            if self.break_time == Self::DOOR_BREAK_TIME
                && self.is_valid_difficulty(world.level_info.load().difficulty)
            {
                world
                    .break_block(&door_pos, None, BlockFlags::SKIP_DROPS)
                    .await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

/// Vanilla: `Vindicator.DOOR_BREAKING_PREDICATE` (`d -> d == Difficulty.NORMAL || d ==
/// Difficulty.HARD`).
#[must_use]
pub const fn normal_or_hard(difficulty: Difficulty) -> bool {
    matches!(difficulty, Difficulty::Normal | Difficulty::Hard)
}

/// Vanilla: `Zombie.DOOR_BREAKING_PREDICATE` (`d -> d == Difficulty.HARD`).
#[must_use]
pub const fn hard_only(difficulty: Difficulty) -> bool {
    matches!(difficulty, Difficulty::Hard)
}

#[cfg(test)]
mod tests {
    use super::{hard_only, normal_or_hard};
    use pumpkin_util::Difficulty;

    #[test]
    fn door_break_predicate_excludes_easy_and_peaceful() {
        assert!(!normal_or_hard(Difficulty::Peaceful));
        assert!(!normal_or_hard(Difficulty::Easy));
        assert!(normal_or_hard(Difficulty::Normal));
        assert!(normal_or_hard(Difficulty::Hard));
    }

    #[test]
    fn zombie_door_break_predicate_is_hard_only() {
        assert!(!hard_only(Difficulty::Peaceful));
        assert!(!hard_only(Difficulty::Easy));
        assert!(!hard_only(Difficulty::Normal));
        assert!(hard_only(Difficulty::Hard));
    }
}
